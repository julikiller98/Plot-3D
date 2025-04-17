fn create_plane_function_material(
    expression: String,

) {
    let wgsl_code = format!(
        r#"
        [[stage(vertex)]]
        fn vertex(
            [[location(0)]] position: vec3<f32>,
            [[binding(0)]] t: f32,
        ) -> [[builtin(position)]] vec4<f32> {{
            let x = position.x;
            let y = position.y;
            let z = {expression};  // Inject user function here
            return vec4<f32>(x, y, z, 1.0);
        }}
        "#,
    );
}