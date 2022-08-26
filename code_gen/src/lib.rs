mod lex;
mod config;

use proc_macro2::TokenStream;
use quote::quote;
use anyhow::Result;

pub struct PipelineConfig {
    module: naga::Module,
}

impl PipelineConfig {
    pub fn from_wgsl(src: &str) -> Result<Self> {
        
        todo!();
    }
}

pub fn gen_pipeline_code(config: &PipelineConfig) -> Result<TokenStream> {
    
    Ok(quote!{
        
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn textured() {
        
    }
}
