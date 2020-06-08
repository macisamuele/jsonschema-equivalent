fn main() {
    if std::env::var("PROFILE") == Ok("debug".to_string()) {
        println!("cargo:rustc-cfg=feature=\"logging\"");
    }
}
