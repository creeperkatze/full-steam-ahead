#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    if cfg!(target_os = "linux")
        && std::env::var_os("GDK_BACKEND").is_none()
        && std::env::var_os("DISPLAY").is_some()
    {
        unsafe { std::env::set_var("GDK_BACKEND", "x11") };
    }

    full_steam_ahead_lib::run();
}
