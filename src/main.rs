#![feature(portable_simd)]

use crate::renderer::RenderPlugin;
use bevy::app::App;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::DefaultPlugins;
use bevy::pbr::{ExtendedMaterial, MaterialPlugin, StandardMaterial};
use bevy_egui::EguiPlugin;
use bevy_flycam::PlayerPlugin;
use crate::wgsl::FunctionMaterial;

mod complex_anita;
mod complex_functions;
mod functions;
mod meshes;
mod renderer;
mod wgsl;

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PlayerPlugin,
            EguiPlugin,
            FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::LogDiagnosticsPlugin::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, FunctionMaterial>>::default(),
            RenderPlugin,
        ))
        .run();
}
