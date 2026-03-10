const TAG: &str = "il2cpp_resolver";

pub fn info(msg: &str) {
    eprintln!("[{}] [INFO] {}", TAG, msg);
}

pub fn error(msg: &str) {
    eprintln!("[{}] [ERROR] {}", TAG, msg);
}

pub fn warning(msg: &str) {
    eprintln!("[{}] [WARN] {}", TAG, msg);
}
