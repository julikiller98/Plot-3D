use bevy::prelude::Mesh;
use bevy::math::Vec3;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::asset::RenderAssetUsages;

pub fn create_tube_mesh<F>(curve_fn: F, segments: usize, radial_segments: usize, radius: f32) -> Mesh
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