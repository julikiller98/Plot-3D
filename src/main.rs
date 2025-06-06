#![feature(portable_simd)]

use crate::renderer::RenderPlugin;
use bevy::app::{App, PluginGroup};
use bevy::asset::AssetApp;
use bevy::asset::io::{AssetSource, AssetSourceId};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::DefaultPlugins;
use bevy::pbr::{ExtendedMaterial, MaterialPlugin, StandardMaterial};
use bevy::prelude::MonitorSelection;
use bevy::utils::default;
use bevy::window::{Window, WindowMode, WindowPlugin};
use bevy_egui::EguiPlugin;
use bevy_flycam::PlayerPlugin;
use crate::wgsl::{CustomShaderProvider, FunctionMaterial};

mod functions;
mod meshes;
mod renderer;
mod wgsl;
pub fn main() {
    App::new()
        .register_asset_source(
            AssetSourceId::from_static("customshader"),
            AssetSource::build().with_reader(|| {Box::new(CustomShaderProvider::new())})
        )
        .add_plugins((
            DefaultPlugins.set(WindowPlugin{
                primary_window: Some(
                    Window{
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                ),
                ..default()
            }),
            PlayerPlugin,
            EguiPlugin,
            FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::LogDiagnosticsPlugin::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, FunctionMaterial>>::default(),
            RenderPlugin,
        ))
        .run();
}
