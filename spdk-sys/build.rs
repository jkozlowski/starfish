extern crate bindgen;
extern crate make_cmd;
extern crate toml;

use std::env;
use std::path::PathBuf;

fn generate_bindings() {
    let spdk_path = env::var("SPDK_DIR").unwrap_or("/tmp/spdk/include".to_string());
    let output_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let generator = Generator { spdk_path, output_path };

    generator.generate("nvme");
    generator.generate("event");
    generator.generate("bdev");
    generator.generate("env");
    generator.generate("blob_bdev");
    generator.generate("blob");
    generator.generate("log");
    generator.generate("io_channel")  
}

struct Generator {
    spdk_path: String,
    output_path: PathBuf
}

impl Generator {
    fn generate(&self, name: &str) {
        let mut codegen_config = bindgen::CodegenConfig::nothing();
        codegen_config.functions = true;
        codegen_config.types = true;

        let bindings = bindgen::Builder::default()
            .header(format!("{}/spdk/{}.h", self.spdk_path, name))
            .derive_default(true)
            .with_codegen_config(codegen_config)
            // Figure out how to make sure the includes are working ok
            .clang_arg(format!("-I{}", self.spdk_path))
            // If there are linking errors and the generated bindings have weird looking
            // #link_names (that start with \u{1}), the make sure to flip that to false.
            .trust_clang_mangling(false)
            .rustfmt_bindings(true)
            .generate()
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        bindings
            .write_to_file(self.output_path.join(format!("spdk_{}_bindings.rs", name)))
            .expect("Couldn't write bindings!");
    }
}

fn main() {
    // Uncomment to regenerate bindings
    generate_bindings();
    println!("cargo:rerun-if-changed=./build.rs");
    println!("cargo:rustc-link-lib=spdk");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
}
