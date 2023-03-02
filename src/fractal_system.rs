use bevy::prelude::*;

/// Render a fractal using a compute shader.
#[derive(Component)]
pub struct ComputeFractalComponent {
    /// What fractal to render.
    pub fractal_type: FractalType,
    /// How many iterations of the fractal set to run.
    pub iterations: usize,
    /// [`Image`] to which the fractal should be drawn.
    pub output: Handle<Image>,
}

/// Types of fractals which can be generated.
#[derive(PartialEq)]
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

pub fn compute_fractal_system(mut _query: Query<&mut ComputeFractalComponent>) {}
