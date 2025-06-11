use std::path::{Path};
use std::sync::{LazyLock, Mutex};
use bevy::asset::{Asset, AssetServer};
use bevy::asset::io::{AssetReader, AssetReaderError, PathStream, Reader};
use bevy::asset::io::memory::{Dir, MemoryAssetReader};
use bevy::pbr::MaterialExtension;
use bevy::prelude::{Entity, Query, Res, TypePath};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use crate::renderer::RenderedFunction;


const CUSTOM_SHADER_NAME: &str  = "shader.wgsl";

const SHADER_ASSET_PATH: &str = "customshader://shader.wgsl";

const SHADER_TEMPLATE_PATH: &str = include_str!("../assets/shaders/shader_template.wgsl");

const FUNCTION_INJECTION_KEY: &str = "{{FUNCTIONS}}";

static POPULATED_TEMPLATE_SHADER: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

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

pub fn rebuild_shader_for_functions(rendered_functions: Query<(Entity, &mut RenderedFunction)>,asset_server: Res<AssetServer>){
    *POPULATED_TEMPLATE_SHADER.lock().expect("could not optain mutex to shader template") = populate_template_shader(rendered_functions);
    asset_server.reload(SHADER_ASSET_PATH);
}

fn populate_template_shader(rendered_functions: Query<(Entity, &mut RenderedFunction)>) -> String {
    let functions = rendered_functions.iter().map(|(_entity, rendered_function)| {
        format!("if id == {} {{ z = {};}}", rendered_function.function_id, rendered_function.function.expression.to_string())
    }).reduce(|a, b| format!("{}\n    {}", a, b)).unwrap_or(String::from(""));
    SHADER_TEMPLATE_PATH.to_string().replace(FUNCTION_INJECTION_KEY, functions.as_str())
}


impl MaterialExtension for FunctionMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

}

pub struct CustomShaderProvider {
    memory_asset_reader: MemoryAssetReader
}

impl CustomShaderProvider {
    pub fn new() -> Self{
        let root= Dir::default();
        root.insert_asset(CUSTOM_SHADER_NAME.as_ref(), Self::populated_shader_bytes());
        CustomShaderProvider{
            memory_asset_reader: MemoryAssetReader{
                root
            }
        }
    }

    fn populated_shader_bytes() -> Vec<u8> {
        let mut populated_shader =  POPULATED_TEMPLATE_SHADER.lock().expect("could not optain mutex to shader template");
        if populated_shader.is_empty() {
            *populated_shader = SHADER_TEMPLATE_PATH.to_string().replace(FUNCTION_INJECTION_KEY, "".as_ref());
        }
        populated_shader.as_bytes().to_vec()
    }
}


impl AssetReader for CustomShaderProvider {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        self.memory_asset_reader.root.insert_asset(path, Self::populated_shader_bytes());
        self.memory_asset_reader.read(path).await
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        self.memory_asset_reader.read_meta(path).await
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        self.memory_asset_reader.read_directory(path).await
    }

    async fn is_directory<'a>(&'a self, path: &'a Path) -> Result<bool, AssetReaderError> {
        self.memory_asset_reader.is_directory(path).await
    }
}