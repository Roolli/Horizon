use futures::{FutureExt, TryFutureExt};

use async_trait::async_trait;
use std::{io::ErrorKind, usize};

use super::fileloader::FileLoader;

#[cfg(target_arch = "wasm32")]
pub struct WebFileLoader {
    base_url: String,
}

#[cfg(target_arch = "wasm32")]
impl WebFileLoader {
    pub fn new(base_path: String) -> Self {
        Self {
            base_url: base_path,
        }
    }
    async fn send_request(&self, path: &str) -> Vec<u8> {
        use js_sys::{ArrayBuffer, Promise};
        use wasm_bindgen::{JsCast, JsValue};
        use wasm_bindgen_futures::{spawn_local, JsFuture};
        let mut opts = web_sys::RequestInit::new();
        opts.method("GET");
        opts.mode(web_sys::RequestMode::Cors);

        let url = format!("{}/{}", self.base_url, path);
        let request: web_sys::Request =
            web_sys::Request::new_with_str_and_init(&url, &opts).unwrap();

        let window = web_sys::window().unwrap();

        let fetch: Promise = window.fetch_with_request(&request);

        let result = JsFuture::from(fetch)
            .map_ok(|response_value| {
                let response: web_sys::Response = response_value.dyn_into().unwrap();
                let array_buffer: Promise = response.array_buffer().unwrap();
                array_buffer
            })
            .and_then(|array_buffer_promise| JsFuture::from(array_buffer_promise))
            .map_ok(|array_buffer: JsValue| {
                let typebuf: js_sys::Uint8Array = js_sys::Uint8Array::new(&array_buffer);
                let mut body = vec![0; typebuf.length() as usize];
                typebuf.copy_to(&mut body[..]);
                body
            });
        let res: Vec<u8> = result.await.unwrap();
        res
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl FileLoader for WebFileLoader {
    ///Fetches the file with the given path as raw bytes
    // TODO: figure something out :-)

    async fn load_file(&self, path: &str) -> Vec<u8> {
        self.send_request(&path).await
    }
}
