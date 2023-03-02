//! Main module.

#![warn(missing_docs)]

mod fractal_system;

use fractal_system::{
    compute_fractal_system, ComputeFractalBundle, ComputeFractalComponent, FractalType,
};

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::window::{PresentMode, WindowDescriptor};

const OUTPUT_RESOLUTION: UVec2 = UVec2 { x: 1280, y: 720 };

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: String::from("Fractal Rendering"),
                present_mode: PresentMode::AutoNoVsync,
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_startup_system(prepare_camera)
        .add_startup_system(prepare_fractal)
        .add_system(compute_fractal_system)
        .run();
}

fn prepare_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn prepare_fractal(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: OUTPUT_RESOLUTION.x,
            height: OUTPUT_RESOLUTION.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );

    // Set texture flags
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let image = images.add(image);

    commands.spawn(ComputeFractalBundle {
        compute_fractal: ComputeFractalComponent {
            fractal_type: FractalType::Julia(-0.45, 0.55),
            iterations: 100,
            output: image.clone(),
        },
        sprite: SpriteBundle {
            sprite: Sprite {
                custom_size: Some(OUTPUT_RESOLUTION.as_vec2()),
                ..default()
            },
            texture: image.clone(),
            ..Default::default()
        },
    });
}
