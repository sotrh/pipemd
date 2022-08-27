mod config;
mod lex;

use std::collections::HashMap;

use anyhow::Result;
use config::{ParseError, RenderPipelineConfig};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub struct PipelineConfig {
    render_configs: Vec<RenderPipelineConfig>,
}

impl PipelineConfig {
    pub fn from_src<'a>(src: &'a str) -> Result<Self, ParseError<'a>> {
        let mut render_configs = Vec::new();
        let mut tokens = lex::TokenStream::new(src)?;

        while let Some(lex::Token::Ident(ident)) = tokens.peek() {
            match ident {
                "render_pipeline" => {
                    render_configs.push(RenderPipelineConfig::parse(&mut tokens)?);
                }
                ident => {
                    return Err(ParseError::UnexpectedToken {
                        found: lex::Token::Ident(ident),
                        expected: lex::Token::Ident("render_pipeline"),
                    })
                }
            }
        }

        Ok(Self { render_configs })
    }
}

pub fn gen_pipeline_code(config: &PipelineConfig) -> Result<TokenStream> {
    struct ShaderData {
        module: naga::Module,
        src: String,
        name: String,
    }
    let mut modules = HashMap::new();
    let mut index = 0;
    let render_pipelines = config.render_configs.iter().map(|rp| {
        let name = format_ident!("{}", rp.name);
        let label = &rp.name;
        let vs_entry = &rp.vs_entry;
        let fs_entry = &rp.fs_entry;

        if !modules.contains_key(&rp.path) {
            let src = std::fs::read_to_string(&rp.path)?;
            let name = format!("SHADER{}", index);
            index += 1;
            let module = naga::front::wgsl::parse_str(&src)?;
            modules.insert(&rp.path, ShaderData { module, src, name });
        }

        let data = &modules[&rp.path];
        let shader_name = &data.name;
        let shader_ident = format_ident!("{}", shader_name);

        Ok(quote! {
            pub struct #name {
                render_pipeline: ::wgpu::RenderPipeline,
            }

            impl #name {
                pub fn new(device: ::wgpu::Device) -> Self {
                    let module = device.create_shader_module(::wgpu::ShaderModuleDescriptor {
                        label: Some(#shader_name),
                        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::from(#shader_ident)),
                    });
                    let pipeline_layout = device.create_pipeline_layout(&::wgpu::PipelineLayoutDescriptor {
                        label: Some(#label),
                        bind_group_layouts: &[],
                        push_constant_ranges: &[],
                    });
                    let render_pipeline = device.create_render_pipeline(&::wgpu::RenderPipelineDescriptor {
                        label: Some(#label),
                        layout: Some(&pipeline_layout),
                        vertex: ::wgpu::VertexState {
                            module: &module,
                            entry_point: #vs_entry,
                            buffers: &[
                                // TODO: pull this data from the module
                            ],
                        },
                        primitive: ::wgpu::PrimitiveState {
                            // TODO: add this data to RenderPipelineConfig
                            topology: ::wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: ::wgpu::FrontFace::Ccw,
                            cull_mode: Some(::wgpu::Face::Back),
                            unclipped_depth: false,
                            polygon_mode: ::wgpu::PolygonMode::Fill,
                            conservative: false,
                        },
                        depth_stencil: None,
                        multisample: ::wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        fragment: Some(::wgpu::FragmentState {
                            module: &module,
                            entry_point: #fs_entry,
                            targets: &[
                                // TODO: pull this data from the module
                            ],
                        }),
                        // Might want to support this 
                        multiview: None,
                    });

                    Self {
                        render_pipeline,
                    }
                }
            }
        })
    }).collect::<Result<Vec<_>>>()?;

    let sources = modules.values().map(|data| {
        let ident = format_ident!("{}", data.name);
        let src = &data.src;
        quote! {
            const #ident: &'static str = #src;
        }
    }).collect::<Vec<_>>();

    Ok(quote! {
        #(#sources)*
        #(#render_pipelines)*
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn pipeline_config_from() {}
}
