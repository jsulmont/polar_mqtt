use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=cpp/impl");
    println!("cargo:rerun-if-changed=cpp/bridge");
    println!("cargo:rerun-if-changed=cpp/api");
    println!("cargo:rerun-if-changed=cpp/CMakeLists.txt");

    let dst = cmake::Config::new("cpp")
        .generator("Unix Makefiles")
        .build_target("all")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("BUILD_SHARED_LIBS", "ON")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .define("CMAKE_INSTALL_RPATH_USE_LINK_PATH", "ON")
        .define("CMAKE_MACOSX_RPATH", "ON")
        .very_verbose(true)
        .build();

    let lib_path = dst.join("build");
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!("cargo:rustc-link-lib=dylib=polar_mqtt_impl");
    println!("cargo:rustc-link-lib=dylib=polar_mqtt_bridge");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../build");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../build");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../build");
    } else {
        panic!("Unsupported OS");
    }

    let bindings = bindgen::Builder::default()
        .header("cpp/bridge/include/mqtt_c.hpp")
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++17")
        .allowlist_file("cpp/bridge/include/mqtt_c.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
