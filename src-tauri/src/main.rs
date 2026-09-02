#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Prevents additional target file to run on the windows platform
fn main() {
    win_optimizer_lib::run()
}