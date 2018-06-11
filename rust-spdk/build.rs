extern crate bindgen;
extern crate make_cmd;
extern crate toml;

use make_cmd::gnu_make;
use std::env;
use std::fmt::Write;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use toml::Value;

// println!("cargo:config_path={}", LLVM_CONFIG_PATH.display()); // will be DEP_LLVM_CONFIG_PATH
// println!("cargo:libdir={}", libdir); // DEP_LLVM_LIBDIR
// println!("cargo:rustc-link-search=native={}", libdir);
// for name in get_link_libraries() {
// println!("cargo:rustc-link-lib=static={}", name);
// }
// println!("cargo:rustc-link-lib=dylib={}", name);
// println!("cargo:rustc-flags={}", cflags)
// # specially recognized by Cargo
// cargo:rustc-link-lib=static=foo
// cargo:rustc-link-search=native=/path/to/foo
// cargo:rustc-cfg=foo
// cargo:rustc-env=FOO=bar
// # arbitrary user-defined metadata
// cargo:root=/path/to/foo
// cargo:libdir=/path/to/foo/lib
// cargo:include=/path/to/foo/include

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("spdk_config.properties");

    let make_output = gnu_make()
        .arg(format!("ENV_PATH={:?}", out_path))
        .output()
        .expect("make failed");

    let mut f = File::open(out_path).expect("file not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    println!("cargo:warn={}", contents);

    let value = contents.parse::<Value>().unwrap();
    let libs = value["LIBS"].as_str().unwrap();

    let mut output = String::new();
    for s in libs.split(" ") {
        write!(&mut output, "\"-C\", \"link-arg={}\",\n", s).unwrap();
    }

    println!("cargo:warn={}", output);
    //let contents = format!("[build]\n", libs)
    //println!("cargo:rustc-flags={}", libs);

    // // Hacks
    // println!("cargo:rustc-link-lib=static=numa");

    // //LINKER_MODULES:
    // //ENV_LIBS:
    // println!("cargo:rustc-link-lib=static=spdk_env_dpdk");
    // println!("cargo:rustc-link-lib=static=rte_eal");
    // println!("cargo:rustc-link-lib=static=rte_mempool");
    // println!("cargo:rustc-link-lib=static=rte_ring");
    // println!("cargo:rustc-link-lib=static=rte_mempool_ring");
    // println!("cargo:rustc-link-lib=static=rte_pci");
    // println!("cargo:rustc-link-lib=static=rte_bus_pci");
    // // Can also use "rustc-flags", but apparently it only supports -l and -L flags.

    // // SPDK_LIB_FILES:
    // println!("cargo:rustc-link-lib=static=spdk_event_bdev");
    // println!("cargo:rustc-link-lib=static=spdk_event_copy");
    // println!("cargo:rustc-link-lib=static=spdk_blobfs");
    // println!("cargo:rustc-link-lib=static=spdk_blob");
    // println!("cargo:rustc-link-lib=static=spdk_bdev");
    // println!("cargo:rustc-link-lib=static=spdk_blob_bdev");
    // println!("cargo:rustc-link-lib=static=spdk_copy");
    // println!("cargo:rustc-link-lib=static=spdk_event");
    // println!("cargo:rustc-link-lib=static=spdk_util");
    // println!("cargo:rustc-link-lib=static=spdk_conf");
    // println!("cargo:rustc-link-lib=static=spdk_trace");
    // println!("cargo:rustc-link-lib=static=spdk_log");
    // println!("cargo:rustc-link-lib=static=spdk_jsonrpc");
    // println!("cargo:rustc-link-lib=static=spdk_json");
    // println!("cargo:rustc-link-lib=static=spdk_rpc");
    // // BLOCKDEV_MODULES_FILES:
    // println!("cargo:rustc-link-lib=static=spdk_vbdev_lvol");
    // println!("cargo:rustc-link-lib=static=spdk_blob");
    // println!("cargo:rustc-link-lib=static=spdk_blob_bdev");
    // println!("cargo:rustc-link-lib=static=spdk_lvol");
    // println!("cargo:rustc-link-lib=static=spdk_bdev_malloc");
    // println!("cargo:rustc-link-lib=static=spdk_bdev_null");
    // println!("cargo:rustc-link-lib=static=spdk_bdev_nvme");
    // println!("cargo:rustc-link-lib=static=spdk_nvme");
    // println!("cargo:rustc-link-lib=static=spdk_vbdev_passthru");
    // println!("cargo:rustc-link-lib=static=spdk_vbdev_error");
    // println!("cargo:rustc-link-lib=static=spdk_vbdev_gpt");
    // println!("cargo:rustc-link-lib=static=spdk_vbdev_split");
    // println!("cargo:rustc-link-lib=static=spdk_bdev_aio");
    // println!("cargo:rustc-link-lib=static=spdk_bdev_virtio");
    // println!("cargo:rustc-link-lib=static=spdk_virtio");

    // println!("cargo:rustc-link-search=native=/usr/local/lib");
    // println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");

    // //println!("cargo:rustc-link-search=/usr/local/lib");

    // Don't rerun the whole thing every time
    println!("cargo:rerun-if-changed=./build.rs");

    generate("nvme");
    generate("event");
    generate("bdev");
    generate("env");
    generate("blob_bdev");
    generate("blob");
    generate("log");
}

fn generate(name: &str) {
    let mut codegen_config = bindgen::CodegenConfig::nothing();
    codegen_config.functions = true;
    codegen_config.types = true;

    let bindings = bindgen::Builder::default()
        .header(format!("/usr/local/include/spdk/{}.h", name))
        .derive_default(true)
        //.whitelist_function("spdk_(env|nvme|dma|mempool).*")
        //.whitelist_type("spdk_(env|nvme|mempool).*")
        .with_codegen_config(codegen_config)
        // Figure out how to make sure the includes are working ok
        .clang_arg("-I/tmp/spdk/include")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join(format!("spdk_{}_bindings.rs", name)))
        .expect("Couldn't write bindings!");
}
