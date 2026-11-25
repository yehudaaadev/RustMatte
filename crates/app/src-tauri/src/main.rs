// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(all(not(debug_assertions), dev, not(test)))]
compile_error!(
    "Release builds of `rustmatte-app` require Tauri's `custom-protocol` feature so embedded frontend assets are served instead of `devUrl` (`http://localhost:1420`). Build with `npx tauri build` inside `crates/app`, or `cargo build --release -p rustmatte-app --features custom-protocol` after `npm run build`."
);

fn main() {
    rustmatte_app_lib::run()
}
