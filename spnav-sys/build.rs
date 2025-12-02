use std::env;
use std::path::PathBuf;

fn main() {
    let library = pkg_config::Config::new()
        .probe("spnav")
        .expect("libspnav not found. Install libspnav-dev or spacenavd-dev, or use the included devenv shell");

    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(
            library
                .include_paths
                .iter()
                .map(|p| format!("-I{}", p.display())),
        )
        .allowlist_function("spnav_.*")
        .allowlist_type("spnav_.*")
        .allowlist_var("SPNAV_.*")
        .rustified_enum("spnav_event_type")
        .layout_tests(true)
        .generate_comments(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
