use bevy::asset::AssetServer;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::{Extract, RenderApp, RenderStage};
use std::borrow::Cow;

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
#[derive(Clone, PartialEq, Reflect)]
#[repr(C)]
pub enum FractalType {
    /// Julia set fractal.
    ///
    /// This type of fractal has two constants used to calculate the next item
    /// of the set.
    Julia(f64, f64),
}

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
            .init_resource::<SpecializedComputePipelines<ComputeFractalPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_fractals)
            .add_system_to_stage(RenderStage::Queue, queue_fractals);
    }
}

#[derive(Resource)]
struct ComputeFractalPipeline {
    shader: Handle<Shader>,
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

        Self {
            shader,
            texture_bind_group_layout,
        }
    }
}

impl SpecializedComputePipeline for ComputeFractalPipeline {
    type Key = ();

    fn specialize(&self, _key: Self::Key) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some(Cow::from("julia_fractal_pipeline")),
            layout: Some(vec![self.texture_bind_group_layout.clone()]),
            shader: self.shader.clone(),
            shader_defs: Vec::new(),
            entry_point: Cow::from("julia"),
        }
    }
}

#[derive(Resource, Default)]
struct ExtractedFractals {
    fractals: Vec<ComputeFractalComponent>,
}

/// Extract the [`ComputeFractalComponents`](ComputeFractalComponent) to the
/// render world.
fn extract_fractals(
    mut extracted_fractals: ResMut<ExtractedFractals>,
    query: Extract<Query<(&ComputeFractalComponent, &ComputedVisibility)>>,
) {
    // Clear the extracted fractals from the last frame
    extracted_fractals.fractals.clear();

    // Find all visible fractals
    for (fractal, visibility) in query.iter() {
        // Fractals dont tend to change much, so we dont need to update
        // it whenever it is out of view.
        if !visibility.is_visible() {
            continue;
        }

        extracted_fractals.fractals.push(fractal.clone());
    }
}

/// Queue the extracted [`ComputeFractalComponents`](ComputeFractalComponent)
/// from the [`extract_fractals`] system in the render graph.
fn queue_fractals(mut commands: Commands, extracted_fractals: Res<ExtractedFractals>) {}
