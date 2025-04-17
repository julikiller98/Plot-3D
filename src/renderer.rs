use crate::complex_anita;
use anita::jit::JITError;
use bevy::asset::RenderAssetUsages;
use bevy::color::palettes::css::WHITE;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::mesh::{CylinderMeshBuilder, Indices, PrimitiveTopology, VertexAttributeValues};
use bevy_egui::egui::{Slider, Widget};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_flycam::PlayerPlugin;
use num_complex::Complex32;
use rand::random;
use std::borrow::Borrow;
use std::default::Default;

pub enum Function {
    StaticComplex(Box<dyn Fn(Complex32) -> Complex32 + Send + Sync>),
    TimedComplex(Box<dyn Fn(Complex32, Complex32) -> Complex32 + Send + Sync>),
    StaticPlane(Box<dyn Fn(Complex32, Complex32) -> Complex32 + Send + Sync>),
    TimedPlane(Box<dyn Fn(Complex32, Complex32, Complex32) -> Complex32 + Send + Sync>),
}

#[derive(Component)]
pub struct TimedFunction;

#[derive(Component)]
pub struct FunctionParsingError(pub JITError);

#[derive(Component)]
pub struct RenderedFunction {
    color: [f32; 3],
    expression: String,
    function: Function,
}
#[derive(Resource)]
pub struct SampleRate(usize);

pub struct MeshBuildOptions {
    pub sample_rate: usize,
    pub time: f32,
}

impl MeshBuildOptions {
    pub fn new(sample_rate: usize, time: f32) -> Self {
        Self {
            sample_rate,
            time
        }
    }
}

const SIZE: f32 = 10.0;
const FUNCTION_RADIUS: f32 = SIZE / 400f32;

const DEFAULT_SAMPLE_RATE: usize = 1000;

impl Function {
    fn build_mesh(&self, options: MeshBuildOptions) -> Mesh {
        let time = options.time.into();
        let sample_rate = options.sample_rate;
        match self {
            Function::StaticComplex(static_function) => create_tube_mesh(
                |x: f32| {
                    let non_normalised_x = (x - 0.5) * SIZE * 2f32;
                    let complex = static_function(non_normalised_x.into());
                    Vec3::new(non_normalised_x, complex.re, complex.im)
                },
                sample_rate,
                32,
                FUNCTION_RADIUS,
            ),
            Function::TimedComplex(timed_function) => create_tube_mesh(
                |x: f32| {
                    let non_normalised_x = (x - 0.5) * SIZE * 2f32;
                    let complex = timed_function(non_normalised_x.into(), time);
                    Vec3::new(non_normalised_x, complex.re, complex.im)
                },
                sample_rate,
                32,
                FUNCTION_RADIUS,
            ),
            Function::StaticPlane(static_plane) => create_surface_mesh(
                |x, y| {
                    let complex = static_plane(x.into(), y.into());
                    Vec3::new(x, complex.re, y)
                },
                sample_rate / 10,
                sample_rate / 10,
                SIZE * 2f32,
            ),
            Function::TimedPlane(timed_plane) => create_surface_mesh(
                |x, y| {
                    let complex = timed_plane(x.into(), y.into(), time);
                    Vec3::new(x, complex.re, y)
                },
                sample_rate / 10,
                sample_rate / 10,
                SIZE * 2f32,
            ),
        }
    }

    fn is_timed(&self) -> bool {
        match self {
            Function::StaticComplex(_) => false,
            Function::StaticPlane(_) => false,
            Function::TimedComplex(_) => true,
            Function::TimedPlane(_) => true,
        }
    }
}

impl RenderedFunction {
    fn material(&self) -> StandardMaterial {

        StandardMaterial {
            base_color: Color::srgb(
                self.color[0],
                self.color[1],
                self.color[2],
            ),
            unlit: match self.function {
                Function::StaticComplex(_) => {true}
                Function::TimedComplex(_) => {true}
                Function::StaticPlane(_) => {false}
                Function::TimedPlane(_) => {false}
            },
            cull_mode: None,
            double_sided: true,
            ..default()
        }
    }
}

fn create_tube_mesh<F>(curve_fn: F, segments: usize, radial_segments: usize, radius: f32) -> Mesh
where
    F: Fn(f32) -> Vec3,
{
    let mut positions = Vec::with_capacity(radial_segments * segments + radial_segments);
    let mut normals = Vec::with_capacity(radial_segments * segments + radial_segments);
    let mut uvs = Vec::with_capacity(radial_segments * segments + radial_segments);
    let mut indices: Vec<u32> = Vec::with_capacity(192 * segments);
    // Pre-calculate points along the curve
    let mut curve_points = Vec::with_capacity(segments + 1);
    let mut tangents = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let point = curve_fn(t);
        curve_points.push(point);
        // Calculate tangent (approximate derivative)
        let delta = 0.001;
        let next_point = curve_fn(t + delta);
        let tangent = (next_point - point).normalize();
        tangents.push(tangent);
    }
    // Generate vertices
    for (segment_idx, (&point, &tangent)) in curve_points.iter().zip(tangents.iter()).enumerate() {
        // Calculate local coordinate system (Frenet frame)
        let normal = if tangent.dot(Vec3::Y) > 0.99 {
            // Handle case where tangent is nearly parallel to up vector
            tangent.cross(Vec3::X).normalize()
        } else {
            tangent.cross(Vec3::Y).normalize()
        };
        let binormal = tangent.cross(normal).normalize();
        // Generate vertices around the circumference
        for radial_idx in 0..radial_segments {
            let angle = 2.0 * std::f32::consts::PI * radial_idx as f32 / radial_segments as f32;
            let radial_dir = normal * angle.cos() + binormal * angle.sin();
            let vertex_pos = point + radial_dir * radius;

            positions.push(vertex_pos.to_array());
            normals.push(radial_dir.to_array());
            uvs.push([
                segment_idx as f32 / segments as f32,
                radial_idx as f32 / radial_segments as f32,
            ]);
        }
    }
    // Generate indices
    for segment_idx in 0..segments {
        for radial_idx in 0..radial_segments {
            let next_radial_idx = (radial_idx + 1) % radial_segments;
            let current = segment_idx * radial_segments + radial_idx;
            let next = segment_idx * radial_segments + next_radial_idx;
            let below = (segment_idx + 1) * radial_segments + radial_idx;
            let below_next = (segment_idx + 1) * radial_segments + next_radial_idx;
            indices.push(current as u32);
            indices.push(below as u32);
            indices.push(next as u32);
            indices.push(next as u32);
            indices.push(below as u32);
            indices.push(below_next as u32);
        }
    }
    // Create the mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    mesh.with_inserted_indices(Indices::U32(indices))
}

fn create_surface_mesh<F>(
    surface_fn: F,
    width_segments: usize,
    height_segments: usize,
    size: f32,
) -> Mesh
where
    F: Fn(f32, f32) -> Vec3,
{
    let required_size = width_segments * height_segments + width_segments + height_segments + 1;
    let mut positions = Vec::with_capacity(required_size);
    let mut normals = Vec::with_capacity(required_size);
    let mut uvs = Vec::with_capacity(required_size);
    let mut indices = Vec::with_capacity(width_segments * height_segments * 6);

    // Generate vertices
    for y in 0..=height_segments {
        let v = y as f32 / height_segments as f32;
        let y_pos = (v - 0.5) * size; // Center around zero

        for x in 0..=width_segments {
            let u = x as f32 / width_segments as f32;
            let x_pos = (u - 0.5) * size; // Center around zero


            // Get position from surface function
            let position = surface_fn(x_pos, y_pos);
            positions.push(position.to_array());

            // UV coordinates (0 to 1)
            uvs.push([u, v]);
        }
    }

    // Calculate normals by computing cross product of adjacent positions
    let mut normals_temp = vec![Vec3::ZERO; positions.len()];

    // Generate indices and compute face normals
    for y in 0..height_segments {
        for x in 0..width_segments {
            let a = y * (width_segments + 1) + x;
            let b = a + 1;
            let c = a + (width_segments + 1);
            let d = c + 1;

            indices.extend_from_slice(&[a as u32, b as u32, d as u32]);
            indices.extend_from_slice(&[a as u32, d as u32, c as u32]);

            // Calculate face normals and accumulate to vertex normals
            if let (Some(a_pos), Some(b_pos), Some(c_pos), Some(d_pos)) = (
                get_vec3(&positions, a),
                get_vec3(&positions, b),
                get_vec3(&positions, c),
                get_vec3(&positions, d),
            ) {
                // First triangle (a, b, d)
                let normal1 = (b_pos - a_pos).cross(d_pos - a_pos).normalize();

                // Second triangle (a, d, c)
                let normal2 = (d_pos - a_pos).cross(c_pos - a_pos).normalize();

                // Accumulate normals
                normals_temp[a] += normal1 + normal2;
                normals_temp[b] += normal1;
                normals_temp[d] += normal1 + normal2;
                normals_temp[c] += normal2;
            }
        }
    }

    // Normalize all normals
    for normal in &mut normals_temp {
        *normal = normal.normalize();
        normals.push(normal.to_array());
    }

    // Create the mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    mesh.with_inserted_indices(Indices::U32(indices))
}
fn get_vec3(positions: &[[f32; 3]], index: usize) -> Option<Vec3> {
    positions.get(index).map(|p| Vec3::new(p[0], p[1], p[2]))
}

pub fn run() -> AppExit {
    use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
    App::new()
        .add_plugins((
            DefaultPlugins,
            PlayerPlugin,
            EguiPlugin,
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .insert_resource(SampleRate(DEFAULT_SAMPLE_RATE))
        .add_systems(Startup, setup)
        .add_systems(Update, (ui_example_system, update_timed_functions))
        .run()
}

fn update_timed_functions(
    mut rendered_functions: Query<(Entity, &RenderedFunction, &Mesh3d), With<(TimedFunction)>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
    sample_rate: Res<SampleRate>,
) {
    for (entity, timed_function, mut mesh) in rendered_functions
        .iter_mut()
    {

        let build_options = MeshBuildOptions::new(
            sample_rate.0,
            time.elapsed_secs(),
        );
        let new_mesh = timed_function
            .function
            .build_mesh(build_options);
        commands.entity(entity).insert(Mesh3d(meshes.add(new_mesh)));
    }
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rendered_functions: Query<(Entity, &mut RenderedFunction, Option<&FunctionParsingError>)>,
    time: Res<Time>,
    mut sample_rate: ResMut<SampleRate>,
) {
    egui::Window::new("Functions").show(contexts.ctx_mut(), |ui| {
        let mut index = 0usize;
        for (entity, mut rendered_function, err) in rendered_functions.iter_mut().sort::<Entity>() {
            ui.horizontal(|ui| {
                ui.add_sized(
                    [30.0, 20.0],
                    egui::Label::new(
                        format!("{:?}(x)= ", (('f' as usize) + index) as u8 as char).replace("'", ""),
                    ),
                );
                index+=1;
                if ui
                    .text_edit_singleline(&mut rendered_function.expression)
                    .changed()
                {
                    append_expression_mesh(
                        &mut commands.entity(entity),
                        &mut meshes,
                        &mut rendered_function,
                        MeshBuildOptions::new(sample_rate.0, time.elapsed_secs()),
                    );
                    commands
                        .entity(entity)
                        .insert(MeshMaterial3d(materials.add(rendered_function.material())));
                }
                if ui
                    .color_edit_button_rgb(&mut rendered_function.color)
                    .changed()
                {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial3d(materials.add(rendered_function.material())));
                }
                if ui.button("X").clicked() {
                    commands.entity(entity).despawn_recursive();
                }
            });
            if let Some(error) = err {
                ui.label(format!("{:?}", error.0));
            }
        }
        if ui.button("+").clicked() {
            add_new_function(&mut commands, &mut meshes, &mut materials, "".to_string())
        }
        ui.label("Sample rate:");
        if Slider::new(&mut sample_rate.0, 100usize..=10000usize)
            .ui(ui)
            .changed()
        {
            for (entity, rendered_function, _) in rendered_functions.iter() {
                rebuild_mesh(
                    &mut commands.entity(entity),
                    &mut meshes,
                    rendered_function,
                    MeshBuildOptions::new(sample_rate.0, time.elapsed_secs())
                );
            }
        }
    });
}

fn append_expression_mesh(
    mut commands: &mut EntityCommands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut rendered_function: &mut RenderedFunction,
    mesh_build_options: MeshBuildOptions
) {
    match complex_anita::parse_anita_fn(rendered_function.expression.clone()) {
        Ok(function) => {
            commands
                .remove::<FunctionParsingError>()
                .insert(Mesh3d(meshes.add(function.build_mesh(
                    mesh_build_options
                ))));
            rendered_function.function = function;
            if rendered_function.function.is_timed() {
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

fn rebuild_mesh(
    commands: &mut EntityCommands,
    meshes: &mut ResMut<Assets<Mesh>>,
    rendered_function: &RenderedFunction,
    mesh_build_options: MeshBuildOptions,
) {
    commands.insert(Mesh3d(
        meshes.add(rendered_function.function.build_mesh(mesh_build_options)),
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
        PointLight {
            intensity: 500_000.0,
            color: WHITE.into(),
            radius: 1.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 10.0, 0.0),
    ));
    add_new_function(&mut commands, &mut meshes, &mut materials, "x;y;t;1".to_string());
}

fn add_new_function(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    expression: String,
) {
    let mut rendered_function = RenderedFunction {
        color: [random(), random(), random()],
        expression,
        function: Function::StaticPlane(Box::new(|x,y| 0f32.into())),
    };
    let mut entity_commands = commands.spawn((
        MeshMaterial3d(materials.add(rendered_function.material())),
    ));
    append_expression_mesh(&mut entity_commands, meshes,&mut rendered_function, MeshBuildOptions::new(DEFAULT_SAMPLE_RATE, 0f32));
    entity_commands.insert(rendered_function);
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
    ));
}
