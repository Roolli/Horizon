use horizon::{run, setup};

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (event_loop, window) = rt.block_on(setup());
    run(event_loop, window, rt);
}
