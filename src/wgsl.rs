use std::fs::File;
use bevy::asset::{Asset, AssetServer, Assets};
use bevy::pbr::{ExtendedMaterial, Material, MaterialExtension, StandardMaterial};
use bevy::prelude::{AlphaMode, Entity, Query, ResMut, Shader, TypePath, With};
use bevy::reflect::erased_serde::__private::serde::de::Unexpected::Str;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use crate::functions::Function;
use crate::renderer::{ChangedExpression, ChangedMaterial, RenderedFunction};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/custom_material.wgsl";

const SHADER_TEMPLATE_PATH: &str = include_str!("../assets/shaders/shader_template.wgsl");

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FunctionMaterial {
    #[uniform(100)]
    pub function_id: u32,
}

pub fn update_function_shader_materials(rendered_functions: Query<(Entity, &mut RenderedFunction)>){
    let functions = rendered_functions.iter().filter(|(e,r)| r.use_shader()).map(|(entity, mut rendered_function)| {
        format!("if id == {} {{ z = {};}}", entity.index(),  rendered_function.expression)
    }).reduce(|a,b| format!("{}\n{}",a,b)).unwrap_or(String::from(""));
    std::fs::write("assets/".to_owned() + SHADER_ASSET_PATH, SHADER_TEMPLATE_PATH.to_string().replace("{{FUNCTIONS}}", functions.as_str())).expect("Unable to create shader template");
}

impl MaterialExtension for FunctionMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

}