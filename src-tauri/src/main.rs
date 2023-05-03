// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  shared::init_env();
  #[cfg(desktop)]
  app_lib::run();
}
