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

pub fn create_surface_mesh<F>(
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