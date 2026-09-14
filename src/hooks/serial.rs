//! # Web Serial Connection Hook
//!
//! ## Overview
//! Manages opening, configuring, reading, and closing browser Web Serial ports, bridging incoming byte streams to Web Workers and UI terminals.
//!
//! ## Search Tags
//! #serial-hook, #web-serial, #port-lifecycle, #stream-reader, #hardware

use crate::hooks::{use_worker_controller, WorkerController};
use crate::state::AppState;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::JsCast;
use web_sys::ReadableStreamDefaultReader;

pub fn use_serial_controller() -> SerialController {
    let state = use_context::<AppState>();
    let bridge = use_worker_controller();

    // Simulation resource
    use_resource(move || {
        let simulating = (state.conn.is_simulating)();
        async move {
            if !simulating {
                return;
            }
            start_simulation_task(state, bridge);
        }
    });

    SerialController { state, bridge }
}

#[derive(Clone, Copy)]
pub struct SerialController {
    state: AppState,
    bridge: WorkerController,
}

impl SerialController {
    pub fn connect(&self) {
        let state = self.state;
        // Prevent action if busy
        if (state.conn.is_busy)() {
            return;
        }
        // Lock immediately to prevent double-click / race conditions
        state.conn.set_busy(true);

        let bridge = self.bridge;
        spawn(async move {
            let Ok(port) = crate::utils::serial_api::request_port().await else {
                state.conn.set_busy(false);
                return;
            };

            if crate::utils::serial_api::open_port(
                &port,
                (state.serial.baud_rate)(),
                (state.serial.data_bits)(),
                (state.serial.stop_bits)(),
                &(state.serial.parity)().to_string(),
                &(state.serial.flow_control)().to_string(),
            )
            .await
            .is_err()
            {
                state.error("Failed to Open Port");
                state.conn.set_busy(false);
                return;
            };

            bridge.new_session();

            // Start the read task explicitly
            start_read_task(state, bridge, port);

            state.success("Connected");
            state.conn.set_busy(false);
        });
    }

    pub fn disconnect(&self) {
        let state = self.state;
        // Prevent action if busy
        if (state.conn.is_busy)() {
            return;
        }
        // Lock immediately to prevent double-click
        state.conn.set_busy(true);

        spawn(async move {
            cleanup_serial_connection(state).await;
            state.info("Disconnected");
            state.conn.set_busy(false);
        });
    }

    pub fn change_baud_rate(&self, new_baud_rate: u32) {
        let state = self.state;
        let bridge = self.bridge;

        if !state.conn.is_connected() {
            state.serial.set_baud_rate(new_baud_rate);
            return;
        }

        if (state.serial.baud_rate)() == new_baud_rate || (state.conn.is_busy)() {
            return;
        }
        state.conn.set_busy(true);

        spawn(async move {
            hot_reopen_serial(state, bridge, new_baud_rate).await;
        });
    }

    pub fn start_simulation(&self) {
        self.state.conn.set_simulating(true);
        { self.state.conn.device_info }.set(Some(crate::utils::SerialDeviceInfo::simulation()));
        self.state.success("Simulation Started");
        self.bridge.clear();
    }

    pub fn stop_simulation(&self) {
        self.state.conn.set_simulating(false);
        self.state.conn.set_reading(false);
        self.state.conn.set_connected(None, None, None);
        self.state.warning("Simulation Stopped");
    }
}

// Helper to cleanup serial connection (Reader + Port) safely
async fn cleanup_serial_connection(state: AppState) {
    // Note: Caller must have set busy=true before calling this

    // Yield to let UI update and previous events settle
    TimeoutFuture::new(50).await;

    let maybe_reader = (state.conn.reader)();
    let maybe_port = (state.conn.port)();

    // 2. Cancel Reader if exists
    if let Some(reader_wrapper) = maybe_reader {
        let _ = crate::utils::serial_api::cancel_reader(&reader_wrapper).await;
    }

    // 3. Wait for Read Loop to finish (Release Lock)
    let mut retries = 0;
    while (state.conn.is_reading)() && retries < 50 {
        TimeoutFuture::new(50).await;
        retries += 1;
    }

    if (state.conn.is_reading)() {
        web_sys::console::warn_1(&"Timeout waiting for reader lock release".into());
    }

    // 4. Close Port if exists

    if let Some(conn_port) = maybe_port {
        if crate::utils::serial_api::close_port(&conn_port)
            .await
            .is_err()
        {
            // Log error if needed, but we proceed to reset state anyway
            web_sys::console::warn_1(&"Failed to close port cleanly".into());
        }
    }

    // 4. Final State Reset
    state.conn.set_connected(None, None, None);
    // state.conn.set_busy(false); // Caller is now responsible for setting busy to false
}

async fn handle_read_completion(
    status: crate::utils::serial_api::ReadStatus,
    state: AppState,
    bridge: WorkerController,
    port: web_sys::SerialPort,
) {
    use crate::utils::serial_api::ReadStatus;

    match status {
        ReadStatus::Retry => {
            TimeoutFuture::new(100).await;
            if (state.conn.is_busy)() {
                return;
            }
            start_read_task(state, bridge, port);
        }
        ReadStatus::Done => {
            if state.conn.is_connected() && !(state.conn.is_busy)() {
                state.conn.set_busy(true);
                state.info("Connection Closed");
                cleanup_serial_connection(state).await;
                state.conn.set_busy(false);
            }
        }
        ReadStatus::Fatal(msg) => {
            if !(state.conn.is_busy)() {
                state.conn.set_busy(true);
                state.error(&format!("Connection Lost: {}", msg));
                cleanup_serial_connection(state).await;
                state.conn.set_busy(false);
            }
        }
    }
}

async fn hot_reopen_serial(state: AppState, bridge: WorkerController, new_baud_rate: u32) {
    TimeoutFuture::new(50).await;

    let maybe_port = (state.conn.port)();
    let maybe_reader = (state.conn.reader)();
    let maybe_device_info = (state.conn.device_info)();

    let Some(port) = maybe_port else {
        state.serial.set_baud_rate(new_baud_rate);
        state.conn.set_busy(false);
        return;
    };

    if let Some(reader) = maybe_reader {
        let _ = crate::utils::serial_api::cancel_reader(&reader).await;
    }

    let mut retries = 0;
    while (state.conn.is_reading)() && retries < 50 {
        TimeoutFuture::new(50).await;
        retries += 1;
    }

    if crate::utils::serial_api::close_port(&port).await.is_err() {
        web_sys::console::warn_1(&"Failed to close port during baud rate change".into());
    }

    state.serial.set_baud_rate(new_baud_rate);

    if crate::utils::serial_api::open_port(
        &port,
        new_baud_rate,
        (state.serial.data_bits)(),
        (state.serial.stop_bits)(),
        &(state.serial.parity)().to_string(),
        &(state.serial.flow_control)().to_string(),
    )
    .await
    .is_err()
    {
        state.error("Failed to reopen port with new baud rate");
        state.conn.set_connected(None, None, None);
        state.conn.set_busy(false);
        return;
    }

    start_read_task_with_info(state, bridge, port, maybe_device_info);
    state.success(&format!("Baud rate changed to {}", new_baud_rate));
    state.conn.set_busy(false);
}

/// Starts an explicit read task that handles the serial read loop and retries
fn start_read_task(state: AppState, bridge: WorkerController, port: web_sys::SerialPort) {
    start_read_task_with_info(state, bridge, port, None);
}

fn start_read_task_with_info(
    state: AppState,
    bridge: WorkerController,
    port: web_sys::SerialPort,
    cached_info: Option<crate::utils::SerialDeviceInfo>,
) {
    spawn(async move {
        let readable = port.readable();
        let reader = readable
            .get_reader()
            .unchecked_into::<ReadableStreamDefaultReader>();

        let device_info = match cached_info {
            Some(info) => info,
            None => crate::utils::get_port_info(&port).await,
        };

        state
            .conn
            .set_connected(Some(port.clone()), Some(reader.clone()), Some(device_info));
        state.conn.set_reading(true);

        let s_clone = state;
        let b_clone = bridge.clone();
        let status = crate::utils::serial_api::read_loop(reader, move |data| {
            if (s_clone.ui.view_mode)() == crate::state::ViewMode::Terminal {
                s_clone.terminal.push_data(data.to_vec());
            } else {
                let is_hex = (s_clone.ui.is_hex_view)();
                b_clone.append_chunk(data, is_hex);
            }
        })
        .await;

        state.conn.set_reading(false);
        handle_read_completion(status, state, bridge, port).await;
    });
}

/// Starts a simulation read task with direct loop cancellation
fn start_simulation_task(state: AppState, bridge: WorkerController) {
    spawn(async move {
        state.conn.set_reading(true);

        while (state.conn.is_simulating)() {
            TimeoutFuture::new(10).await;
            if !(state.conn.is_simulating)() {
                break;
            }

            let chunk = crate::utils::simulation::generate_simulated_chunk();
            if (state.ui.view_mode)() == crate::state::ViewMode::Terminal {
                state.terminal.push_data(chunk.to_vec());
            } else {
                let is_hex = (state.ui.is_hex_view)();
                bridge.append_chunk(chunk, is_hex);
            }
        }

        state.conn.set_reading(false);
    });
}
