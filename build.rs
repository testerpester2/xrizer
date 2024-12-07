fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    for path in shaders::compile(&out_dir) {
        println!("cargo::rerun-if-changed={}", path.to_str().unwrap());
    }
}
