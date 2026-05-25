fn main() {
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
    // Don't unwrap if file doesn't exist so the user can just set env vars at build time
    println!("cargo:rerun-if-changed=buildtime-vars");
    if let Ok(creds_lines) = std::fs::read_to_string("buildtime-vars") {
        for line in creds_lines.lines() {
            if !line.is_empty() {
                let val_pair = line.trim_start().trim_end();
                println!("cargo:rustc-env={val_pair}")
            }
        }
    }
    #[cfg(not(feature = "ntp"))]
    {
        // pass the time via env vars

        use chrono::Utc;
        let local_time = Utc::now();
        let local_time_str = local_time.to_rfc3339();
        println!("cargo:rustc-env=BUILD_DATETIME={local_time_str}");
    }
}
