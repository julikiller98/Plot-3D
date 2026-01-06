#![feature(portable_simd)]

use crate::editor::FunctionEditorPlugin;
use crate::player::PlayerPlugin;
use crate::renderer::RenderPlugin;
use crate::wgsl::FunctionMaterial;
use bevy::app::{App, PluginGroup};
use bevy::color::Color;
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig};
use bevy::pbr::{ExtendedMaterial, MaterialPlugin, StandardMaterial};
use bevy::prelude::{MonitorSelection, TextFont};
use bevy::text::FontSmoothing;
use bevy::utils::default;
use bevy::window::{Window, WindowMode, WindowPlugin};
use bevy::DefaultPlugins;

mod functions;
mod renderer;
mod wgsl;
mod expression;
mod editor;
mod player;

pub fn main() {
    App::new()
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
                    frame_time_graph_config: FrameTimeGraphConfig{
                        enabled: false,
                        ..default()
                    },
                    ..default()
                },
            },
            PlayerPlugin,
            bevy_egui::EguiPlugin::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, FunctionMaterial>>::default(),
            RenderPlugin,
            FunctionEditorPlugin
        ))
        .run();
}
