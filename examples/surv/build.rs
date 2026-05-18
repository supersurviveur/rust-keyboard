fn main() {
    if std::path::Path::new("./src/private.rs").exists() {
        println!("cargo:rustc-cfg=surv_private")
    }
}
