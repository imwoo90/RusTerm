use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::prelude::*;
use wasm_streams::ReadableStream;

pub fn create_simulation_stream() -> web_sys::ReadableStream {
    let stream = futures_util::stream::unfold((), |()| async move {
        TimeoutFuture::new(10).await; // Using 10ms to prevent overwhelming the UI, can be adjusted.

        let rnd = js_sys::Math::random();
        let content = if rnd < 0.1 {
            format!("Error: System overheat at {:.1}°C\n", 80.0 + rnd * 20.0)
        } else if rnd < 0.3 {
            format!("Warning: Voltage fluctuation detected: {:.2}V\n", 3.0 + rnd)
        } else {
            format!(
                "Info: Sensor reading: A={:.2}, B={:.2}, C={:.2}\n",
                rnd * 100.0,
                rnd * 50.0,
                rnd * 10.0
            )
        };

        let chunk = js_sys::Uint8Array::from(content.as_bytes());
        // Stream expects Result<JsValue, JsValue>
        Some((Ok(JsValue::from(chunk)), ()))
    });

    ReadableStream::from_stream(stream).into_raw()
}
