fn main() {
    if std::env::var("PROFILE").as_deref() == Ok("debug") {
        println!("cargo:rustc-cfg=dev_release");
    }
}
