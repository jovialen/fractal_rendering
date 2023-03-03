//! Bevy fractal system with compute shaders

use bevy::asset::AssetServer;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::{Extract, RenderApp, RenderStage};
use std::borrow::Cow;
use std::collections::HashMap;

/// Render a fractal using a compute shader.
#[derive(Component, Clone, Reflect)]
#[repr(C)]
pub struct ComputeFractalComponent {
    /// What fractal to render.
    pub fractal_type: FractalType,
    /// How many iterations of the fractal set to run.
    pub iterations: usize,
    /// [`Image`] to which the fractal should be drawn.
    pub output: Handle<Image>,
}

/// Types of fractals which can be generated.
#[derive(Clone, Copy, PartialEq, Reflect)]
#[repr(C)]
pub enum FractalType {
    /// Julia set fractal.
    ///
    /// This type of fractal has two constants used to calculate the next item
    /// of the set.
    Julia(f64, f64),
}

/// Bundle with everything needed to create an entity with a compute fractal
/// rendered to a sprite.
#[derive(Bundle)]
pub struct ComputeFractalBundle {
    pub compute_fractal: ComputeFractalComponent,

    #[bundle]
    pub sprite: SpriteBundle,
}

/// System to render a fractal on its output image.
pub struct ComputeFractalPlugin;

impl Plugin for ComputeFractalPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ComputeFractalComponent>();

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ExtractedFractals>()
            .init_resource::<ComputeFractalPipeline>()
            .add_system_to_stage(RenderStage::Extract, extract_fractals)
            .add_system_to_stage(RenderStage::Queue, queue_fractals);
    }
}

/// Compute pipeline for all fractal generation.
#[derive(Resource)]
struct ComputeFractalPipeline {
    /// Compute pipeline for the julia fractal.
    julia_pipeline: CachedComputePipelineId,
    /// Common bind group for all pipelines.
    texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for ComputeFractalPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut system_state: SystemState<(Res<RenderDevice>, Res<AssetServer>)> =
            SystemState::new(world);
        let (render_device, asset_server) = system_state.get_mut(world);

        let texture_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("compute_fractal_texture_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        let shader = asset_server.load("shaders/fractal_system.wgsl");

        let mut pipeline_cache = world.resource_mut::<PipelineCache>();
        let julia_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("julia_fractal_pipeline")),
            layout: Some(vec![texture_bind_group_layout.clone()]),
            shader: shader.clone(),
            shader_defs: Vec::new(),
            entry_point: Cow::from("julia"),
        });

        Self {
            julia_pipeline,
            texture_bind_group_layout,
        }
    }
}

/// A fractal extracted from the logical ecs world to the render world.
struct ExtractedFractal {
    entity: Entity,
    fractal_type: FractalType,
    iterations: usize,
    output: Handle<Image>,
}

/// All fractals to be processed by the renderer.
#[derive(Resource, Default)]
struct ExtractedFractals {
    fractals: Vec<ExtractedFractal>,
}

/// Extract the [`ComputeFractalComponents`](ComputeFractalComponent) to the
/// render world.
fn extract_fractals(
    mut extracted_fractals: ResMut<ExtractedFractals>,
    query: Extract<Query<(Entity, &ComputeFractalComponent, &ComputedVisibility)>>,
) {
    // Clear the extracted fractals from the last frame
    extracted_fractals.fractals.clear();

    // Find all visible fractals
    for (entity, fractal, visibility) in query.iter() {
        // Fractals dont tend to change much, so we dont need to update
        // it whenever it is out of view.
        if !visibility.is_visible() {
            continue;
        }

        extracted_fractals.fractals.push(ExtractedFractal {
            entity,
            fractal_type: fractal.fractal_type,
            iterations: fractal.iterations,
            output: fractal.output.clone_weak(),
        });
    }
}

/// Queue the extracted [`ComputeFractalComponents`](ComputeFractalComponent)
/// from the [`extract_fractals`] system in the render graph.
fn queue_fractals(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<Image>>,
    pipeline: Res<ComputeFractalPipeline>,
    extracted_fractals: Res<ExtractedFractals>,
) {
    let mut bind_groups = ComputeFractalBindGroups::default();

    for fractal in extracted_fractals.fractals.iter() {
        // Get a texture view of the fractal output image
        let view = &gpu_images[&fractal.output];

        // Create a compatible bind group with the texture view
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.texture_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view.texture_view),
            }],
        });

        bind_groups.values.insert(fractal.entity, bind_group);
    }

    commands.insert_resource(bind_groups);
}

/// Bind groups for all extracted fractals.
#[derive(Default, Resource)]
struct ComputeFractalBindGroups {
    values: HashMap<Entity, BindGroup>,
}
