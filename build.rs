fn main() {
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
    #[cfg(feature = "networking")]
    {
        // Don't unwrap if file doesn't exist so the user can just set env vars at build time
        if let Ok(creds_lines) = std::fs::read_to_string("wifi-creds") {
            for line in creds_lines.lines() {
                if !line.is_empty() {
                    let val_pair = line.trim_start().trim_end();
                    println!("cargo:rustc-env={val_pair}")
                }
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
