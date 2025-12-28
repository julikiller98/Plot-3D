use crate::functions::Function;
use crate::wgsl::{setup_dynamic_shader, FunctionMaterial};
use crate::{functions, wgsl};
use bevy::pbr::{ExtendedMaterial, NotShadowCaster};
use bevy::prelude::*;
use bevy::render::mesh::CylinderMeshBuilder;
use bevy::render::view::NoFrustumCulling;
use rand::random;

const SIZE: f32 = 10.0;
const DEFAULT_RESOLUTION: usize = 10000;

#[derive(Component)]
pub struct RenderedFunction {
    pub(crate) color: [f32; 3],
    pub(crate) expression: String,
    pub (crate) function: Function,
    pub(crate) function_id: u32,
}
#[derive(Resource)]
pub struct Scale(pub f32);

impl Default for Scale {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Resource, Default)]
pub struct OffsetX(pub f32);

#[derive(Resource, Default)]
pub struct OffsetY(pub f32);

#[derive(Resource)]
pub struct Resolution(pub usize);
impl Default for Resolution {
    fn default() -> Self {
        Self(DEFAULT_RESOLUTION)
    }
}

#[derive(Component)]
pub struct FunctionParsingError(pub String);

#[derive(Component)]
pub struct ChangedMaterial;

#[derive(Component)]
pub struct ChangedExpression;

#[derive(Component)]
pub struct RebuildMesh;

#[derive(Resource)]
pub struct UpdateShader(pub(crate) bool);

#[derive(Component)]
pub struct Axis;

#[derive(Resource)]
pub struct RenderAxis(pub(crate) bool);

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
}

impl MeshBuildOptions {
    pub fn new(sample_rate: usize) -> Self {
        Self { sample_rate }
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
        let subdivisions = (options.sample_rate / 10).max(1) as u32;
        Plane3d::new(-Dir3::Y.as_vec3(), Vec2::new(SIZE, SIZE))
            .mesh()
            .subdivisions(subdivisions)
            .build()
    }
}

fn check_axis_visibility(render_axis: Res<RenderAxis>,mut commands: Commands, axis: Query<Entity, With<Axis>>) {
    for entity in axis.iter() {
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
    for (entity, rendered_function) in rendered_functions.iter_mut() {
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
    resolution: ResMut<Resolution>,
    mut update_shader: ResMut<UpdateShader>,
) {
    for (entity, mut rendered_function) in rendered_functions.iter_mut() {
        update_shader.0 = update_shader.0 | append_expression_mesh(
            &mut commands.entity(entity).remove::<ChangedExpression>(),
            &mut meshes,
            &mut rendered_function,
            MeshBuildOptions::new(resolution.0),
        );
    }
}

fn rebuild_changed_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rendered_functions: Query<(Entity, &mut RenderedFunction), With<RebuildMesh>>,
    resolution: ResMut<Resolution>,
) {
    for (entity, rendered_function) in rendered_functions.iter_mut() {
        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(rendered_function.build_mesh(
                MeshBuildOptions::new(resolution.0),
            ))))
            .remove::<RebuildMesh>();
    }
}


fn update_shader(
    mut update_shader: ResMut<UpdateShader>,
    rendered_functions: Query<& RenderedFunction>,
    shaders: ResMut<Assets<Shader>>
) {
    if update_shader.0 {
        wgsl::rebuild_shader_for_functions(rendered_functions, shaders);
        update_shader.0 = false;
    }
}

fn append_expression_mesh(
    commands: &mut EntityCommands,
    meshes: &mut ResMut<Assets<Mesh>>,
    rendered_function: &mut RenderedFunction,
    mesh_build_options: MeshBuildOptions,
) -> bool{
    let last_expression = rendered_function.function.expression.clone();
    match functions::parse_anita_fn(rendered_function.expression.clone()) {
        Ok(function) => {
            commands.remove::<FunctionParsingError>();
            if function.expression == last_expression {
                return false;
            }
            rendered_function.function = function;
            commands.insert(Mesh3d(
                meshes.add(rendered_function.build_mesh(mesh_build_options)),
            ));
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
        NoFrustumCulling
    ));
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Resolution>()
            .init_resource::<OffsetX>()
            .init_resource::<OffsetY>()
            .init_resource::<Scale>()
            .insert_resource(UpdateShader(true))
            .insert_resource(RenderAxis(false))
            .insert_resource(FunctionIdCounter(1))
            .add_systems(Startup, setup)
            .add_systems(Startup, setup_dynamic_shader)
            .add_systems(
                Update,
                (
                    recompile_changed_expressions,
                    recompile_changed_materials,
                    rebuild_changed_meshes,
                    check_axis_visibility,
                    update_shader,
                ),
            );
    }
}
