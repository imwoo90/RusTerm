//! # Serial Device Port Resolver
//!
//! ## Overview
//! Queries authorized browser serial ports to disambiguate identical chipsets, resolves vendor information, and manages user alias storage.
//!
//! ## Search Tags
//! #resolver, #port-disambiguation, #user-alias, #web-serial

use super::{identify_device, SerialDeviceInfo};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[allow(dead_code)]
pub fn get_stored_alias(
    vid: Option<u16>,
    pid: Option<u16>,
    port_index: Option<usize>,
) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let storage = web_sys::window()?.local_storage().ok()??;
        let key = format!(
            "rusterm_port_alias_{:04X}_{:04X}_{}",
            vid.unwrap_or(0),
            pid.unwrap_or(0),
            port_index.unwrap_or(1)
        );
        storage.get_item(&key).ok().flatten()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (vid, pid, port_index);
        None
    }
}

pub fn set_stored_alias(
    vid: Option<u16>,
    pid: Option<u16>,
    port_index: Option<usize>,
    alias: &str,
) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                let key = format!(
                    "rusterm_port_alias_{:04X}_{:04X}_{}",
                    vid.unwrap_or(0),
                    pid.unwrap_or(0),
                    port_index.unwrap_or(1)
                );
                if alias.trim().is_empty() {
                    let _ = storage.remove_item(&key);
                } else {
                    let _ = storage.set_item(&key, alias.trim());
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (vid, pid, port_index, alias);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn extract_vid_pid(obj: &wasm_bindgen::JsValue) -> (Option<u16>, Option<u16>) {
    let info_obj = js_sys::Reflect::get(obj, &"getInfo".into())
        .ok()
        .and_then(|f| {
            if f.is_function() {
                let func: js_sys::Function = f.unchecked_into();
                func.call0(obj).ok()
            } else {
                None
            }
        });

    if let Some(ref info) = info_obj {
        let v = js_sys::Reflect::get(info, &"usbVendorId".into())
            .ok()
            .and_then(|val| val.as_f64().map(|n| n as u16));
        let p = js_sys::Reflect::get(info, &"usbProductId".into())
            .ok()
            .and_then(|val| val.as_f64().map(|n| n as u16));
        (v, p)
    } else {
        (None, None)
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn find_port_index_and_total(
    target_port: &wasm_bindgen::JsValue,
    target_vid: Option<u16>,
    target_pid: Option<u16>,
) -> (Option<usize>, usize) {
    let Some(win) = web_sys::window() else { return (None, 0); };
    let Ok(ports_val) = JsFuture::from(win.navigator().serial().get_ports()).await else {
        return (None, 0);
    };
    let Ok(ports_arr) = ports_val.dyn_into::<js_sys::Array>() else { return (None, 0); };

    let mut match_count = 0;
    let mut resolved_index = None;
    let len = ports_arr.length();

    for i in 0..len {
        let p = ports_arr.get(i);
        let is_same = p == *target_port;
        let (vid, pid) = extract_vid_pid(&p);

        if vid == target_vid && pid == target_pid {
            match_count += 1;
            if is_same {
                resolved_index = Some(match_count);
            }
        }
    }

    (resolved_index, match_count)
}

pub fn build_device_info(
    vid: Option<u16>,
    pid: Option<u16>,
    port_index: Option<usize>,
    total_ports: usize,
    alias: Option<String>,
) -> SerialDeviceInfo {
    let mut dev = identify_device(vid, pid);
    dev.port_index = port_index;
    dev.alias = alias.clone();

    let base_label = dev.label.clone();
    let base_desc = dev.description.clone();

    if let Some(ref a) = alias {
        if !a.trim().is_empty() {
            if let Some(idx) = port_index {
                dev.label = format!("{a} [{base_label} #{idx}]");
                dev.description = format!("{a} ({base_desc}, Port #{idx})");
            } else {
                dev.label = format!("{a} [{base_label}]");
                dev.description = format!("{a} ({base_desc})");
            }
            return dev;
        }
    }

    if let Some(idx) = port_index {
        if total_ports > 1 || idx > 1 {
            dev.label = format!("{base_label} #{idx}");
            dev.description = format!("{base_desc} (Port #{idx})");
        }
    }

    dev
}

pub async fn resolve_port_info(port: &web_sys::SerialPort) -> SerialDeviceInfo {
    #[cfg(target_arch = "wasm32")]
    {
        let js_val: &wasm_bindgen::JsValue = port.as_ref();
        let (vid, pid) = extract_vid_pid(js_val);
        let (port_index, total_ports) = find_port_index_and_total(js_val, vid, pid).await;
        let alias = get_stored_alias(vid, pid, port_index);
        build_device_info(vid, pid, port_index, total_ports, alias)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = port;
        build_device_info(None, None, None, 0, None)
    }
}
