//! Hardware serial device identification via USB Vendor ID (VID) and Product ID (PID).
//!
//! Provides lookup tables and metadata resolvers for common USB-to-UART bridge chipsets
//! (FTDI, Silicon Labs, WCH, Espressif, Raspberry Pi, Arduino, STMicroelectronics, etc.).

mod vendors;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct SerialDeviceInfo {
    pub label: String,
    pub description: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
}

impl SerialDeviceInfo {
    pub fn simulation() -> Self {
        Self {
            label: "Simulation".to_string(),
            description: "Simulated Serial Stream (Test Mode)".to_string(),
            vid: None,
            pid: None,
        }
    }

    pub fn fallback() -> Self {
        Self {
            label: "Connected".to_string(),
            description: "Connected Serial Port".to_string(),
            vid: None,
            pid: None,
        }
    }
}

pub fn identify_device(vid: Option<u16>, pid: Option<u16>) -> SerialDeviceInfo {
    if let Some(v) = vid {
        if let Some((label, desc)) = vendors::lookup_known_vendor(v, pid) {
            return SerialDeviceInfo {
                label: label.to_string(),
                description: desc.to_string(),
                vid,
                pid,
            };
        }

        if let Some(p) = pid {
            SerialDeviceInfo {
                label: format!("USB [{:04X}:{:04X}]", v, p),
                description: format!("USB Serial Device (VID: 0x{:04X}, PID: 0x{:04X})", v, p),
                vid,
                pid,
            }
        } else {
            SerialDeviceInfo {
                label: format!("USB [{:04X}]", v),
                description: format!("USB Serial Device (VID: 0x{:04X})", v),
                vid,
                pid,
            }
        }
    } else {
        SerialDeviceInfo::fallback()
    }
}

pub fn get_port_info(port: &web_sys::SerialPort) -> SerialDeviceInfo {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let js_val: &wasm_bindgen::JsValue = port.as_ref();
        let info_obj = js_sys::Reflect::get(js_val, &"getInfo".into())
            .ok()
            .and_then(|get_info_fn| {
                if get_info_fn.is_function() {
                    let func: js_sys::Function = get_info_fn.unchecked_into();
                    func.call0(js_val).ok()
                } else {
                    None
                }
            });

        let (vid, pid) = if let Some(ref info) = info_obj {
            let v = js_sys::Reflect::get(info, &"usbVendorId".into())
                .ok()
                .and_then(|val| val.as_f64().map(|n| n as u16));
            let p = js_sys::Reflect::get(info, &"usbProductId".into())
                .ok()
                .and_then(|val| val.as_f64().map(|n| n as u16));
            (v, p)
        } else {
            (None, None)
        };

        identify_device(vid, pid)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = port;
        identify_device(None, None)
    }
}
