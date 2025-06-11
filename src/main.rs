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
use crate::editor::FunctionEditorPlugin;
use crate::player::PlayerPlugin;
use crate::wgsl::{CustomShaderProvider, FunctionMaterial};

mod functions;
mod renderer;
mod wgsl;
mod expression;
mod editor;
mod player;

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
                        font_size: 10.0,
                        font: default(),
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    text_color: Color::WHITE,
                    enabled: true,
                    ..default()
                },
            },
            PlayerPlugin,
            bevy_egui::EguiPlugin{ enable_multipass_for_primary_context: false },
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, FunctionMaterial>>::default(),
            RenderPlugin,
            FunctionEditorPlugin
        ))
        .run();
}
