extern crate bindgen;
extern crate gcc;
extern crate pkg_config;

use gcc::Config;
use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

fn main() {
//    if cfg!(target_os = "linux") {
//        let out_dir = env::var("OUT_DIR").unwrap();
//        let dest_path = Path::new(&out_dir);
//
//        generate_bindgen("/usr/include/libaio.h", &dest_path, "aio".to_string());
//
//        println!("cargo:rustc-flags=-l aio");
//
//        gcc::Config::new()
//            .file("src/file/aio_macros.c")
//            .include("/usr/include/libaio.h")
//            .include("/usr/include/xfs/xfs.h")
//            .include("/usr/include/xfs/linux.h")
//            .include("/use/include/xfs/xfs_fs.h")
//            .compile("libaio_macros.a");
//    }

    return;
}

fn generate_bindgen<T: Into<String>>(path: T, out_dir: &Path, mod_name: String) {
    let aio_abi = bindgen::Builder::new(path).generate().expect("Failed to generate bindings");

    let dest_path = out_dir.join(format!("{}.rs", mod_name));
    let mut file = File::create(&dest_path).expect("Failed to open file");
    // Wrap the bindings in a `pub mod` before writing bindgen's output
    file.write(format!("pub mod {} {{\n", mod_name).as_bytes()).unwrap();
    file.write(aio_abi.to_string().as_bytes()).unwrap();
    file.write(b"}").unwrap();
}
