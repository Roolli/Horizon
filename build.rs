use deno_core::{JsRuntime, RuntimeOptions};
use std::env;
use std::fs::{read_to_string, write};
use std::path::PathBuf;
pub struct TimerPermission;

impl deno_web::TimersPermission for TimerPermission {
    fn allow_hrtime(&mut self) -> bool {
        true
    }

    fn check_unstable(&self, state: &deno_core::OpState, api_name: &'static str) {}
}
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let o = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let snapshot_path = o.join("HORIZON_SNAPSHOT.bin");
    let options = RuntimeOptions {
        will_snapshot: true,
        extensions: vec![
            deno_webidl::init(),
            deno_console::init(),
            deno_url::init(),
            deno_web::init::<TimerPermission>(deno_web::BlobStore::default(), None),
        ],
        ..Default::default()
    };
    let mut isolate = deno_core::JsRuntime::new(options);
    let snapshot = isolate.snapshot();
    let snapshot_slice: &[u8] = &*snapshot;
    std::fs::write(&snapshot_path, &snapshot_slice).unwrap();
    println!("Snapshot written to: {}", snapshot_path.display());
}
