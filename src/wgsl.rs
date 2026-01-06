use bevy::asset::{uuid_handle};
use bevy::prelude::*;
use bevy::pbr::MaterialExtension;
use bevy::render::render_resource::{AsBindGroup};
use bevy::shader::ShaderRef;
use crate::renderer::RenderedFunction;

pub const DYNAMIC_SHADER_HANDLE: Handle<Shader> = uuid_handle!("35f8c75b-73ef-49af-b50c-920d48c19ef3");
const SHADER_TEMPLATE: &str = include_str!("../assets/shaders/shader_template.wgsl");
const FUNCTION_INJECTION_KEY: &str = "{{FUNCTIONS}}";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FunctionMaterial {

    #[uniform(100)]
    pub function_id: u32,
    #[uniform(100)]
    pub scale: f32,
    #[uniform(100)]
    pub offset_x: f32,
    #[uniform(100)]
    pub offset_y: f32,
}

impl MaterialExtension for FunctionMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(DYNAMIC_SHADER_HANDLE)
    }

}

pub fn rebuild_shader_for_functions(
    rendered_functions: Query<&RenderedFunction>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    let mut function_body = String::new();
    function_body.push_str("switch id {\n");
    for func in rendered_functions.iter() {
        function_body.push_str(format!(
            "    case {}u: {{ z = {}; }}\n",
            func.function_id,
            func.function.expression.to_string()
        ).as_str());
    }
    function_body.push_str("    default: { z = 0.0; }\n");
    function_body.push_str("}\n");

    let final_code = SHADER_TEMPLATE.replace(FUNCTION_INJECTION_KEY, &function_body);
    let shader = Shader::from_wgsl(final_code, "dynamic_shader.wgsl");
    let _ = shaders.insert(DYNAMIC_SHADER_HANDLE.id(), shader);
}

pub fn setup_dynamic_shader(mut shaders: ResMut<Assets<Shader>>) {
    let initial_code = SHADER_TEMPLATE.replace(FUNCTION_INJECTION_KEY, "z = 0.0;");
    let shader = Shader::from_wgsl(initial_code, "dynamic_shader.wgsl");
    let _ = shaders.insert(DYNAMIC_SHADER_HANDLE.id(), shader);
}