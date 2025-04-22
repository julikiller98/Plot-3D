use crate::functions::Function;
use crate::wgsl::FunctionMaterial;
use crate::{functions, meshes, wgsl};
use anita::jit::JITError;
use bevy::pbr::{ExtendedMaterial, NotShadowCaster};
use bevy::prelude::*;
use bevy::render::mesh::CylinderMeshBuilder;
use bevy_egui::egui::{Slider, Widget};
use bevy_egui::{egui, EguiContexts};
use rand::random;
use std::borrow::Borrow;
use std::default::Default;
use std::hash::Hash;

const SIZE: f32 = 10.0;
const FUNCTION_RADIUS: f32 = SIZE / 400f32;
const DEFAULT_SAMPLE_RATE: usize = 1000;

#[derive(Component)]
pub struct RenderedFunction {
    color: [f32; 3],
    pub(crate) expression: String,
    function: Function,
}

#[derive(Component)]
pub struct TimedFunction;

#[derive(Component)]
pub struct FunctionParsingError(pub JITError);

#[derive(Component)]
pub struct ChangedMaterial;

#[derive(Component)]
pub struct ChangedExpression;

#[derive(Component)]
pub struct RebuildMesh;

#[derive(Resource)]
pub struct SampleRate(usize);

#[derive(Resource)]
pub struct UpdateShader(bool);

#[derive(Component)]
pub struct Axis;

#[derive(Resource)]
pub struct RenderAxis(bool);

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
            unlit: match self.function {
                Function::StaticComplex(_) => true,
                Function::TimedComplex(_) => true,
                Function::StaticPlane(_) => false,
                Function::TimedPlane(_) => false,
            },
            cull_mode: None,
            reflectance: 0.0,
            double_sided: true,
            ..default()
        }
    }

    pub fn use_shader(&self) -> bool {
        match self.function {
            Function::StaticComplex(_) => false,
            Function::TimedComplex(_) => false,
            Function::StaticPlane(_) => true,
            Function::TimedPlane(_) => true,
        }
    }

    fn build_mesh(&self, options: MeshBuildOptions) -> Mesh {
        let time = options.time.into();
        let sample_rate = options.sample_rate;
        match &self.function {
            Function::StaticComplex(static_function) => meshes::create_tube_mesh(
                |x: f32| {
                    let non_normalised_x = (x - 0.5) * SIZE * 2f32;
                    let complex = static_function(non_normalised_x.into());
                    Vec3::new(non_normalised_x, complex.re, complex.im)
                },
                sample_rate,
                32,
                FUNCTION_RADIUS,
            ),
            Function::TimedComplex(timed_function) => meshes::create_tube_mesh(
                |x: f32| {
                    let non_normalised_x = (x - 0.5) * SIZE * 2f32;
                    let complex = timed_function(non_normalised_x.into(), time);
                    Vec3::new(non_normalised_x, complex.re, complex.im)
                },
                sample_rate,
                32,
                FUNCTION_RADIUS,
            ),
            Function::StaticPlane(_) | Function::TimedPlane(_) => {
                Plane3d::new(-Dir3::Y.as_vec3(), Vec2::new(SIZE, SIZE))
                    .mesh()
                    .subdivisions(sample_rate as u32 / 10)
                    .build()
            }
        }
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

fn update_timed_functions(
    mut rendered_functions: Query<(Entity, &RenderedFunction, &Mesh3d), With<(TimedFunction)>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
    sample_rate: Res<SampleRate>,
) {
    for (entity, timed_function, _) in rendered_functions.iter_mut() {
        let build_options = MeshBuildOptions::new(sample_rate.0, time.elapsed_secs());
        let new_mesh = timed_function.build_mesh(build_options);
        commands.entity(entity).insert(Mesh3d(meshes.add(new_mesh)));
    }
}

fn recompile_changed_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, FunctionMaterial>>>,
    mut rendered_functions: Query<(Entity, &mut RenderedFunction), With<ChangedMaterial>>,
) {
    for (entity, mut rendered_function) in rendered_functions.iter_mut() {
        commands
            .entity(entity)
            .insert(MeshMaterial3d(materials.add(ExtendedMaterial {
                base: rendered_function.material(),
                extension: FunctionMaterial {
                    function_id: entity.index(),
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
    mut sample_rate: ResMut<SampleRate>,
    mut update_shader: ResMut<UpdateShader>,
) {
    let mut force_shader_reload = false;
    for (entity, mut rendered_function) in rendered_functions.iter_mut() {
        append_expression_mesh(
            &mut commands.entity(entity).remove::<ChangedExpression>(),
            &mut meshes,
            &mut rendered_function,
            MeshBuildOptions::new(sample_rate.0, time.elapsed_secs()),
        );
        force_shader_reload = force_shader_reload || rendered_function.use_shader();
    }
    if force_shader_reload {
        update_shader.0 = true;
    }
}

fn rebuild_changed_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rendered_functions: Query<(Entity, &mut RenderedFunction), With<RebuildMesh>>,
    time: Res<Time>,
    mut sample_rate: ResMut<SampleRate>,
) {
    for (entity, mut rendered_function) in rendered_functions.iter_mut() {
        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(rendered_function.build_mesh(
                MeshBuildOptions::new(sample_rate.0, time.elapsed_secs()),
            ))))
            .remove::<RebuildMesh>();
    }
}

fn update_shader(
    mut update_shader: ResMut<UpdateShader>,
    rendered_functions: Query<(Entity, &mut RenderedFunction)>,
) {
    if update_shader.0 {
        wgsl::rebuild_shader_for_functions(rendered_functions);
        update_shader.0 = false;
    }
}

fn function_editor(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut rendered_functions: Query<(Entity, &mut RenderedFunction, Option<&FunctionParsingError>)>,
    mut sample_rate: ResMut<SampleRate>,
    mut reload_shaders: ResMut<UpdateShader>,
    mut render_axis: ResMut<RenderAxis>,
) {
    egui::Window::new("Functions").show(contexts.ctx_mut(), |ui| {
        let mut index = 0usize;
        for (entity, mut rendered_function, err) in rendered_functions.iter_mut().sort::<Entity>() {
            ui.horizontal(|ui| {
                ui.add_sized(
                    [30.0, 20.0],
                    egui::Label::new(
                        format!("{:?}(x)= ", (('f' as usize) + index) as u8 as char)
                            .replace("'", ""),
                    ),
                );
                index += 1;
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
                    function: Function::StaticPlane(Box::new(|x, y| 0f32.into())),
                },
                ChangedMaterial,
                ChangedExpression,
                NotShadowCaster,
            ));
        }
        ui.label("Sample rate:");
        if Slider::new(&mut sample_rate.0, 100usize..=10000usize)
            .ui(ui)
            .changed()
        {
            for (entity, _, _) in rendered_functions.iter() {
                commands.entity(entity).insert(RebuildMesh);
            }
        }

        ui.checkbox(&mut render_axis.0, "Show Axis: ");
    });
}

fn append_expression_mesh(
    mut commands: &mut EntityCommands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut rendered_function: &mut RenderedFunction,
    mesh_build_options: MeshBuildOptions,
) {
    match functions::parse_anita_fn(rendered_function.expression.clone()) {
        Ok(function) => {
            rendered_function.function = function;
            commands.remove::<FunctionParsingError>().insert(Mesh3d(
                meshes.add(rendered_function.build_mesh(mesh_build_options)),
            ));
            if rendered_function.function.rebuild_on_update() {
                commands.insert(TimedFunction);
            } else {
                commands.remove::<TimedFunction>();
            }
            commands.remove::<FunctionParsingError>();
        }
        Err(err) => {
            commands.insert(FunctionParsingError(err));
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
    let default_fn_expression = "-x ** x".to_string();
    commands.spawn((
        RenderedFunction {
            function: functions::parse_anita_fn(default_fn_expression.clone()).expect("default function should be valid"),
            expression: default_fn_expression,
            color: [random(), random(), random()],
        },
        ChangedExpression,
        ChangedMaterial,
        NotShadowCaster,
    ));
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SampleRate(DEFAULT_SAMPLE_RATE))
            .insert_resource(UpdateShader(true))
            .insert_resource(RenderAxis(true))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    function_editor,
                    update_timed_functions,
                    recompile_changed_expressions,
                    recompile_changed_materials,
                    rebuild_changed_meshes,
                    update_shader,
                    check_axis_visibility
                ),
            );
    }
}
