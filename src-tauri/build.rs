use std::env;

fn main() {
    // Emit the OUT_DIR environment variable for Tauri's generate_context!()
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| "target/debug".to_string());
    println!("cargo:rustc-env=TAURI_PLATFORM_TYPE={}", "desktop");
    println!("cargo:rustc-env=TAURI_OUT_DIR={}", out_dir);
    println!("cargo:rustc-env=TAURI_PLATFORM={}", std::env::consts::OS);
}
