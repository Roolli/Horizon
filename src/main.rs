#![windows_subsystem = "windows"]

#[cfg(not(target_arch = "wasm32"))]
use horizon::{run, setup};
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (event_loop, window) = rt.block_on(setup());
    run(event_loop, window, rt);
}
#[cfg(target_arch = "wasm32")]
fn main() {}
