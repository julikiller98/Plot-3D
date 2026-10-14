import bevy_pbr::render::{
    mesh_functions,
    mesh_view_bindings::globals,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
    mesh_bindings::mesh,
    skinning,
    morph,
};

struct FunctionMaterial {
    function_id: u32,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<uniform> function_material: FunctionMaterial;

@if(MORPH_TARGETS)
fn morph_vertex(vertex_in: Vertex) -> Vertex {
    var vertex = vertex_in;
    let first_vertex = mesh[vertex.instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;

    let weight_count = morph::layer_count();
    for (var i: u32 = 0u; i < weight_count; i++) {
        let weight = morph::weight_at(i);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph::morph(vertex_index, morph::position_offset, i);
        @if(VERTEX_NORMALS) {
            vertex.normal += weight * morph::morph(vertex_index, morph::normal_offset, i);
        }
        @if(VERTEX_TANGENTS) {
            vertex.tangent += vec4(weight * morph::morph(vertex_index, morph::tangent_offset, i), 0.0);
        }
    }
    return vertex;
}

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var vertex_shifted = vertex_no_morph;
    vertex_shifted.position.y = vertex_shifted.position.y + f(vertex_shifted.position.x, vertex_shifted.position.z);
    vertex_shifted.normal = calc_normal(vertex_shifted.position.x, vertex_shifted.position.z);

    var vertex = vertex_shifted;
    @if(MORPH_TARGETS) {
        vertex = morph_vertex(vertex);
    }

    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);

    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var world_from_local = mesh_world_from_local;
    @if(SKINNED) {
        world_from_local = skinning::skin_model(
            vertex.joint_indices,
            vertex.joint_weights,
            vertex_no_morph.instance_index
        );
    }

    @if(VERTEX_NORMALS) {
        @if(SKINNED) {
            out.world_normal = skinning::skin_normals(world_from_local, vertex.normal);
        }
        @else {
            out.world_normal = mesh_functions::mesh_normal_local_to_world(
                vertex.normal,
                vertex_no_morph.instance_index
            );
        }
    }

    @if(VERTEX_POSITIONS) {
        out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
        out.position = position_world_to_clip(out.world_position.xyz);
    }

    @if(VERTEX_UVS_A) {
        out.uv = vertex.uv;
    }
    @if(VERTEX_UVS_B) {
        out.uv_b = vertex.uv_b;
    }

    @if(VERTEX_TANGENTS) {
        out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
            world_from_local,
            vertex.tangent,
            vertex_no_morph.instance_index
        );
    }

    @if(VERTEX_COLORS) {
        out.color = vertex.color;
    }

    @if(VERTEX_OUTPUT_INSTANCE_INDEX) {
        out.instance_index = vertex_no_morph.instance_index;
    }

    @if(VISIBILITY_RANGE_DITHER) {
        out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
            vertex_no_morph.instance_index, mesh_world_from_local[3]);
    }

    return out;
}

const pi = 3.14159265358979323846264338327950288;
const e = 2.71828182845904523536028747135266250;

fn f(ix: f32, iy: f32) -> f32 {
    let t = globals.time;
    let id = function_material.function_id;
    let x = function_material.scale * (ix + function_material.offset_x);
    let y = function_material.scale * (iy + function_material.offset_y);
    var z = 0.0;
    {{FUNCTIONS}}
    return z;
}

fn calc_normal(x: f32, y: f32) -> vec3<f32> {
    const dx = 0.001;
    const dy = 0.001;
    let f0 = f(x, y);
    let tx = vec3f(dx, f(x + dx, y) - f0, 0.0);
    let ty = vec3f(0.0, f(x, y + dy) - f0, dy);
    return normalize(cross(tx, ty));
}