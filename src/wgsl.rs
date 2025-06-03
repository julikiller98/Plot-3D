use std::fs::File;
use std::path::PathBuf;
use bevy::asset::{Asset, AssetEvent, AssetServer, Assets};
use bevy::asset::io::AssetSourceEvent;
use bevy::pbr::{ExtendedMaterial, Material, MaterialExtension, StandardMaterial};
use bevy::prelude::{AlphaMode, Entity, EventWriter, Query, Res, ResMut, Shader, TypePath, With};
use bevy::reflect::erased_serde::__private::serde::de::Unexpected::Str;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use crate::functions::Function;
use crate::renderer::{ChangedExpression, ChangedMaterial, RenderedFunction};

const SHADER_ASSET_PATH: &str = "shaders/custom_material.wgsl";

const SHADER_TEMPLATE_PATH: &str = include_str!("../assets/shaders/shader_template.wgsl");

const FUNCTION_INJECTION_KEY: &str = "{{FUNCTIONS}}";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FunctionMaterial {
    #[uniform(100)]
    pub function_id: u32,
}

pub fn rebuild_shader_for_functions(rendered_functions: Query<(Entity, &mut RenderedFunction)>,asset_server: Res<AssetServer>, asset_event_writer: &mut EventWriter<AssetEvent<Shader>>){
    let functions = rendered_functions.iter().filter(|(e,r)| r.use_shader()).map(|(entity, mut rendered_function)| {
        format!("if id == {} {{ z = {};}}", entity.index(),  rendered_function.expression)
    }).reduce(|a,b| format!("{}\n    {}",a,b)).unwrap_or(String::from(""));
    std::fs::write("assets/".to_owned() + SHADER_ASSET_PATH, SHADER_TEMPLATE_PATH.to_string().replace(FUNCTION_INJECTION_KEY, functions.as_str())).expect("Unable to write custom shader!");
    asset_event_writer.send(AssetEvent::Modified{id: asset_server.load(SHADER_ASSET_PATH).id()});
}

impl MaterialExtension for FunctionMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

}