extern crate bindgen;

use std::env;
use std::io::{stderr, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::{exit, Command};

// fn exec(command_name: &str, mut cmd: Command) {
//     match cmd.output() {
//         Ok(out) => if !out.stdout(Stdio::inherit()).status.success() {
//             let _ = writeln!(
//                 &mut stderr(),
//                 "{} failed:\n {}",
//                 command_name,
//                 String::from_utf8(out.stderr).unwrap()
//             );
//             exit(1);
//         },
//         Err(e) => {
//             let _ = writeln!(&mut stderr(), "{} exec failed: {:?}", command_name, e);
//             exit(1);
//         }
//     }
// }

fn main() {
    // let mut config_spdk = Command::new("./configure");
    // config_spdk.current_dir(Path::new("spdk"));
    // exec("spdk config", config_spdk);

    // let mut make_spdk = Command::new("make");
    // make_spdk.current_dir(Path::new("spdk"));
    // exec("spdk make", make_spdk);
    println!("Path: {:?}", env::var("OUT_DIR"));

    //println!("cargo:rustc-link-lib=static=spdk_env_dpdk");
    //println!("cargo:rustc-link-lib=static=spdk_log");
    //println!("cargo:rustc-link-lib=static=spdk_util");
    //println!("cargo:rustc-link-lib=static=spdk_nvme");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    //println!("cargo:rustc-link-search=spdk/build/lib");
    //println!("cargo:rustc-link-lib=spdk/dpdk");
    //println!("cargo:rustc-link-search=spdk/dpdk/x86_64-native-linuxapp-gcc/lib");

    let mut codegen_config = bindgen::CodegenConfig::nothing();
    codegen_config.functions = true;
    codegen_config.types = true;

    let bindings = bindgen::Builder::default()
        .header("spdk/include/spdk/nvme.h")
        .derive_default(true)
        // .whitelisted_type("SomeCoolClass")
        .whitelist_function("spdk_(env|nvme|dma|mempool).*")
        .whitelist_type("spdk_(env|nvme|mempool).*")
        .with_codegen_config(codegen_config)
        .clang_arg("-Ispdk/include")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("spdk_bindings.rs"))
        .expect("Couldn't write bindings!");

    // --with-derive-default --whitelist-function "spdk_(env|nvme|dma|mempool).*" \
    //     --whitelist-type "spdk_(env|nvme|mempool).*" --generate functions,types  -- -Ispdk/include
}
