use crate::functions::Function;
use crate::wgsl::FunctionMaterial;
use crate::{functions, meshes, wgsl};
use bevy::pbr::{ExtendedMaterial, NotShadowCaster};
use bevy::prelude::*;
use bevy::render::mesh::CylinderMeshBuilder;
use bevy_egui::egui::{Slider, Widget};
use bevy_egui::{egui, EguiContexts};
use rand::random;
use std::borrow::Borrow;
use std::default::Default;
use std::hash::Hash;
use crate::expression::Expr;
use crate::expression::Expr::Exp;

const SIZE: f32 = 10.0;
const FUNCTION_RADIUS: f32 = SIZE / 400f32;
const DEFAULT_RESOLUTION: usize = 10000;

#[derive(Component)]
pub struct RenderedFunction {
    color: [f32; 3],
    pub(crate) expression: String,
    pub (crate) function: Function,
    pub(crate) function_id: u32,
}
#[derive(Resource)]
pub struct Scale(pub f32);

#[derive(Resource)]
pub struct OffsetX(pub f32);

#[derive(Resource)]
pub struct OffsetY(pub f32);

#[derive(Resource)]
pub struct Resolution(pub usize);


#[derive(Component)]
pub struct FunctionParsingError(pub String);

#[derive(Component)]
pub struct ChangedMaterial;

#[derive(Component)]
pub struct ChangedExpression;

#[derive(Component)]
pub struct RebuildMesh;

#[derive(Resource)]
pub struct UpdateShader(bool);

#[derive(Component)]
pub struct Axis;

#[derive(Resource)]
pub struct RenderAxis(bool);

#[derive(Resource)]

pub struct FunctionIdCounter(u32);

impl FunctionIdCounter {
    pub fn next_id(&mut self) -> u32 {
        self.0 += 1;
        self.0
    }
}

pub struct MeshBuildOptions {
    pub sample_rate: usize,
    pub time: f32,
}

impl MeshBuildOptions {
    pub fn new(sample_rate: usize, time: f32) -> Self {
        Self { sample_rate, time }
    }
}

impl RenderedFunction {
    fn material(&self) -> StandardMaterial {
        StandardMaterial {
            base_color: Color::srgb(self.color[0], self.color[1], self.color[2]),
            unlit: false,
            cull_mode: None,
            reflectance: 0.0,
            double_sided: true,
            ..default()
        }
    }


    fn build_mesh(&self, options: MeshBuildOptions) -> Mesh {
        let sample_rate = options.sample_rate;
        Plane3d::new(-Dir3::Y.as_vec3(), Vec2::new(SIZE, SIZE))
            .mesh()
            .subdivisions(sample_rate as u32 / 10)
            .build()
    }
}

fn check_axis_visibility(render_axis: Res<RenderAxis>,mut commands: Commands, axis: Query<(Entity), With<Axis>>) {
    for (entity) in axis.iter() {
        if render_axis.0 {
            commands.entity(entity).insert(Visibility::Visible);
        } else {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
}


fn recompile_changed_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FunctionMaterial>>>,
    mut rendered_functions: Query<(Entity, &mut RenderedFunction), With<ChangedMaterial>>,
    offset_x: Res<OffsetX>,
    offset_y: Res<OffsetY>,
    scale: Res<Scale>
) {
    for (entity, mut rendered_function) in rendered_functions.iter_mut() {
        commands
            .entity(entity)
            .insert(MeshMaterial3d(materials.add(ExtendedMaterial {
                base: rendered_function.material(),
                extension: FunctionMaterial {
                    function_id: rendered_function.function_id,
                    scale: scale.0,
                    offset_x: offset_x.0,
                    offset_y: offset_y.0,
                },
            })))
            .remove::<ChangedMaterial>();
    }
}

fn recompile_changed_expressions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rendered_functions: Query<(Entity, &mut RenderedFunction), With<ChangedExpression>>,
    time: Res<Time>,
    mut resolution: ResMut<Resolution>,
    mut update_shader: ResMut<UpdateShader>,
) {
    for (entity, mut rendered_function) in rendered_functions.iter_mut() {
        update_shader.0 = update_shader.0 | append_expression_mesh(
            &mut commands.entity(entity).remove::<ChangedExpression>(),
            &mut meshes,
            &mut rendered_function,
            MeshBuildOptions::new(resolution.0, time.elapsed_secs()),
        );
    }
}

fn rebuild_changed_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rendered_functions: Query<(Entity, &mut RenderedFunction), With<RebuildMesh>>,
    time: Res<Time>,
    mut resolution: ResMut<Resolution>,
) {
    for (entity, mut rendered_function) in rendered_functions.iter_mut() {
        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(rendered_function.build_mesh(
                MeshBuildOptions::new(resolution.0, time.elapsed_secs()),
            ))))
            .remove::<RebuildMesh>();
    }
}


fn update_shader(
    mut update_shader: ResMut<UpdateShader>,
    asset_server: Res<AssetServer>,
    rendered_functions: Query<(Entity, &mut RenderedFunction)>,
) {
    if update_shader.0 {
        wgsl::rebuild_shader_for_functions(rendered_functions, asset_server);
        update_shader.0 = false;
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
    mut function_id_counter: ResMut<FunctionIdCounter>
) {
    egui::Window::new("Functions").show(contexts.ctx_mut(), |ui| {
        for (entity, mut rendered_function, err) in rendered_functions.iter_mut().sort_by::<&RenderedFunction>(
            |a,b| a.function_id.cmp(&b.function_id)
        ){
            ui.horizontal(|ui| {
                ui.add_sized(
                    [30.0, 20.0],
                    egui::Label::new(
                        format!("{:?}(x)= ", (('f' as u32) + rendered_function.function_id) as u8 as char)
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
                    commands.entity(entity).despawn_recursive();
                    reload_shaders.0 = true;
                }
            });
            if let Some(error) = err {
                ui.label(format!("{:?}", error.0));
            }
        }
        if ui.button("+").clicked() {
            commands.spawn((
                RenderedFunction {
                    color: [random(), random(), random()],
                    expression: "".to_string(),
                    function: Function{
                        expression: Expr::parse_string("x".to_string()).unwrap()
                    },
                    function_id: function_id_counter.next_id(),
                },
                ChangedMaterial,
                ChangedExpression,
                NotShadowCaster,
            ));
        }
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Scale:");
                ui.add_space(4.8);
                ui.label("Resolution:");
                ui.add_space(4.8);
                ui.label("Offset x:");
                ui.add_space(4.8);
                ui.label("Offset y:");
            });
            ui.vertical(|ui| {
                if Slider::new(&mut scale.0, 0f32..=10f32)
                    .ui(ui)
                    .changed()
                {
                    for (entity, _, _) in rendered_functions.iter() {
                        commands.entity(entity).insert(ChangedMaterial);
                    }
                }
                if Slider::new(&mut resolution.0, 100usize..=10000usize)
                    .ui(ui)
                    .changed()
                {
                    for (entity, _, _) in rendered_functions.iter() {
                        commands.entity(entity).insert(RebuildMesh);
                    }
                }
                if Slider::new(&mut offset_x.0, -10f32..=10f32)
                    .ui(ui)
                    .changed() ||
                    Slider::new(&mut offset_y.0, -10f32..=10f32)
                        .ui(ui)
                        .changed()
                {
                    for (entity, _, _) in rendered_functions.iter() {
                        commands.entity(entity).insert(ChangedMaterial);
                    }
                }
            })
        }
        );
        ui.checkbox(&mut render_axis.0, "Show Axis: ");
    });
}

fn append_expression_mesh(
    mut commands: &mut EntityCommands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut rendered_function: &mut RenderedFunction,
    mesh_build_options: MeshBuildOptions,
) -> bool{
    let last_expression = rendered_function.function.expression.clone();
    match functions::parse_anita_fn(rendered_function.expression.clone()) {
        Ok(function) => {
            if function.expression == last_expression {
                return false;
            }
            rendered_function.function = function;
            commands.remove::<FunctionParsingError>().insert(Mesh3d(
                meshes.add(rendered_function.build_mesh(mesh_build_options)),
            ));
            commands.remove::<FunctionParsingError>();
            true
        }
        Err(err) => {
            commands.insert(FunctionParsingError(err));
            false
        }
    }
}

fn setup_axis(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    dir: Vec3,
    splits: usize,
) {
    let part = dir / splits as f32;
    for split in 0..splits {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.01f32 * dir.length()))),
            Transform::from_translation(part * split as f32),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.5, 1.0),
                unlit: true,
                ..default()
            })),
            NotShadowCaster,
            Axis
        ));
    }
    commands.spawn((
        Mesh3d(meshes.add(CylinderMeshBuilder::new(
            0.0025 * dir.length(),
            dir.length(),
            32,
        ))),
        Transform::from_rotation(Quat::from_rotation_arc(Vec3::Y, dir.normalize()))
            .with_translation(dir / 2f32),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        })),
        NotShadowCaster,
        Axis
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.02f32 * dir.length()))),
        Transform::from_translation(dir),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 1.0),
            unlit: true,
            ..default()
        })),
        NotShadowCaster,
        Axis,
    ));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut function_id_counter: ResMut<FunctionIdCounter>
) {
    for axis in [Vec3::X, Vec3::Y, Vec3::Z, -Vec3::X, -Vec3::Y, -Vec3::Z] {
        setup_axis(&mut commands, &mut meshes, &mut materials, axis * SIZE, 10);
    }
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    let default_fn_expression_0 = "e ** (x * sin(y))".to_string();
    let default_fn_expression_1 = "e ** (x * sin(y + t))".to_string();
    commands.spawn((
        RenderedFunction {
            function: functions::parse_anita_fn(default_fn_expression_0.clone()).expect("default function should be valid"),
            expression: default_fn_expression_1,
            color: [random(), random(), random()],
            function_id: function_id_counter.next_id(),
        },
        ChangedExpression,
        ChangedMaterial,
        NotShadowCaster,
    ));
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Resolution(DEFAULT_RESOLUTION))
            .insert_resource(OffsetX(0f32))
            .insert_resource(OffsetY(0f32))
            .insert_resource(Scale(1f32))
            .insert_resource(UpdateShader(true))
            .insert_resource(RenderAxis(false))
            .insert_resource(FunctionIdCounter(1))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    function_editor,
                    recompile_changed_expressions,
                    recompile_changed_materials,
                    rebuild_changed_meshes,
                    update_shader,
                    check_axis_visibility
                ),
            );
    }
}
