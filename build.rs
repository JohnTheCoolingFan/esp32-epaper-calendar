fn main() {
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
    let creds_lines = std::fs::read_to_string("wifi-creds").unwrap();
    for line in creds_lines.lines() {
        if !line.is_empty() {
            let val_pair = line.trim_start().trim_end();
            println!("cargo:rustc-env={val_pair}")
        }
    }
}
