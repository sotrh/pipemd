use std::fs::read_to_string;
use std::io::Write;

use code_gen;

#[cfg(test)]
mod tests {
    use code_gen::PipelineConfig;
    use quote::quote;

    use super::*;

    #[test]
    fn textured() {
        let src = read_to_string("./tests/texture.pmd").unwrap();
        let config = PipelineConfig::from_src(&src).unwrap();
        let pipeline_code = code_gen::gen_pipeline_code(&config).unwrap();
        let tokens = quote!{
            #pipeline_code

            fn main() {}
        };

        let mut file = std::fs::File::create("./tests/temp/texture.rs").unwrap();
        write!(file, "{}", tokens).unwrap();

        let tests = trybuild::TestCases::new();
        tests.pass("./tests/temp/texture.rs");
    }
}