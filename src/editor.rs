use crate::expression::Expr;
use crate::functions::Function;
use crate::renderer::{
    ChangedExpression, ChangedMaterial, FunctionIdCounter, FunctionParsingError, OffsetX, OffsetY,
    RebuildMesh, RenderAxis, RenderedFunction, Resolution, Scale, UpdateShader,
};
use bevy::app::{App, Plugin, Update};
use bevy::pbr::NotShadowCaster;
use bevy::prelude::{Commands, Entity, Mut, Query, ResMut};
use bevy_egui::egui::{Slider, Ui, Widget};
use bevy_egui::{egui, EguiContexts};
use rand::random;
use std::ops::RangeInclusive;
use bevy::render::view::NoFrustumCulling;

pub struct FunctionEditorPlugin;

const SCALE_RANGE: RangeInclusive<f32> = 0.1..=10.0;
const RESOLUTION_RANGE: RangeInclusive<usize> = 100usize..=10000usize;

const OFFSET_RANGE: RangeInclusive<f32> = -10f32..=10f32;

impl Plugin for FunctionEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, function_editor);
    }
}

fn function_editor(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut rendered_functions: Query<(Entity, &mut RenderedFunction, Option<&FunctionParsingError>)>,
    mut resolution: ResMut<Resolution>,
    mut offset_x: ResMut<OffsetX>,
    mut offset_y: ResMut<OffsetY>,
    mut scale: ResMut<Scale>,
    mut reload_shaders: ResMut<UpdateShader>,
    mut render_axis: ResMut<RenderAxis>,
    function_id_counter: ResMut<FunctionIdCounter>,
) {
    egui::Window::new("Functions").show(contexts.ctx_mut(), |ui| {
        edit_functions_ui(
            &mut commands,
            &mut rendered_functions,
            &mut reload_shaders,
            ui,
        );
        add_function_button_ui(&mut commands, function_id_counter, ui);
        config_ui(
            commands,
            rendered_functions,
            &mut resolution,
            &mut offset_x,
            &mut offset_y,
            &mut scale,
            &mut render_axis,
            ui,
        );
    });
}

fn edit_functions_ui(
    mut commands: &mut Commands,
    rendered_functions: &mut Query<(
        Entity,
        &mut RenderedFunction,
        Option<&FunctionParsingError>,
    )>,
    mut reload_shaders: &mut ResMut<UpdateShader>,
    ui: &mut Ui,
) {
    for (entity, mut rendered_function, err) in rendered_functions
        .iter_mut()
        .sort_by::<&RenderedFunction>(|a, b| a.function_id.cmp(&b.function_id))
    {
        edit_function_ui(
            &mut commands,
            &mut reload_shaders,
            ui,
            entity,
            &mut rendered_function,
        );
        if let Some(error) = err {
            ui.label(format!("{:?}", error.0));
        }
    }
}

fn add_function_button_ui(
    mut commands: &mut Commands,
    function_id_counter: ResMut<FunctionIdCounter>,
    ui: &mut Ui,
) {
    if ui.button("+").clicked() {
        add_empty_function(&mut commands, function_id_counter);
    }
}

fn add_empty_function(
    commands: &mut &mut Commands,
    mut function_id_counter: ResMut<FunctionIdCounter>,
) {
    commands.spawn((
        RenderedFunction {
            color: [random(), random(), random()],
            expression: "".to_string(),
            function: Function {
                expression: Expr::parse_string("x".to_string()).unwrap(),
            },
            function_id: function_id_counter.next_id(),
        },
        ChangedMaterial,
        ChangedExpression,
        NotShadowCaster,
        NoFrustumCulling
    ));
}

fn config_ui(
    mut commands: Commands,
    rendered_functions: Query<(Entity, &mut RenderedFunction, Option<&FunctionParsingError>)>,
    resolution: &mut ResMut<Resolution>,
    offset_x: &mut ResMut<OffsetX>,
    offset_y: &mut ResMut<OffsetY>,
    scale: &mut ResMut<Scale>,
    render_axis: &mut ResMut<RenderAxis>,
    ui: &mut Ui,
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("Resolution:");
            ui.add_space(4.8);
            ui.label("Scale:");
            ui.add_space(4.8);
            ui.label("Offset x:");
            ui.add_space(4.8);
            ui.label("Offset y:");
        });
        ui.vertical(|ui| {
            if Slider::new(&mut resolution.0, RESOLUTION_RANGE)
                .ui(ui)
                .changed()
            {
                for (entity, _, _) in rendered_functions.iter() {
                    commands.entity(entity).insert(RebuildMesh);
                }
            }
            if Slider::new(&mut scale.0, SCALE_RANGE).ui(ui).changed()
                | Slider::new(&mut offset_x.0, OFFSET_RANGE).ui(ui).changed()
                | Slider::new(&mut offset_y.0, OFFSET_RANGE).ui(ui).changed()
            {
                for (entity, _, _) in rendered_functions.iter() {
                    commands.entity(entity).insert(ChangedMaterial);
                }
            }
        })
    });
    ui.checkbox(&mut render_axis.0, "Show Axis: ");
}

fn edit_function_ui(
    commands: &mut Commands,
    reload_shaders: &mut ResMut<UpdateShader>,
    ui: &mut Ui,
    entity: Entity,
    rendered_function: &mut Mut<RenderedFunction>,
) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [30.0, 20.0],
            egui::Label::new(
                format!(
                    "{:?}(x)= ",
                    (('f' as u32) + rendered_function.function_id) as u8 as char
                )
                .replace("'", ""),
            ),
        );
        if ui
            .text_edit_singleline(&mut rendered_function.expression)
            .changed()
        {
            commands.entity(entity).insert(ChangedExpression);
        }
        if ui
            .color_edit_button_rgb(&mut rendered_function.color)
            .changed()
        {
            commands.entity(entity).insert(ChangedMaterial);
        }
        if ui.button("X").clicked() {
            commands.entity(entity).despawn();
            reload_shaders.0 = true;
        }
    });
}
