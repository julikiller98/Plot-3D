#![feature(portable_simd)]

use crate::renderer::RenderPlugin;
use bevy::app::{App, PluginGroup};
use bevy::asset::AssetApp;
use bevy::asset::io::{AssetSource, AssetSourceId};
use bevy::color::Color;
use bevy::DefaultPlugins;
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::pbr::{ExtendedMaterial, MaterialPlugin, StandardMaterial};
use bevy::prelude::{MonitorSelection, TextFont};
use bevy::text::FontSmoothing;
use bevy::utils::default;
use bevy::window::{Window, WindowMode, WindowPlugin};
use bevy_egui::EguiPlugin;
use bevy_flycam::PlayerPlugin;
use crate::wgsl::{CustomShaderProvider, FunctionMaterial};

mod functions;
mod meshes;
mod renderer;
mod wgsl;
mod expression;

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
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 10.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::WHITE,
                    // We can also set the refresh interval for the FPS counter
                    enabled: true,
                },
            },
            PlayerPlugin,
            EguiPlugin,
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, FunctionMaterial>>::default(),
            RenderPlugin,
        ))
        .run();
}
