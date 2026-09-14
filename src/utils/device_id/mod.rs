//! Hardware serial device identification via USB Vendor ID (VID) and Product ID (PID).
//!
//! Provides lookup tables, multi-device disambiguation, and user alias persistence for serial ports.

mod resolver;
mod vendors;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use resolver::build_device_info;
pub use resolver::set_stored_alias;

#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct SerialDeviceInfo {
    pub label: String,
    pub description: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub port_index: Option<usize>,
    pub alias: Option<String>,
}

impl SerialDeviceInfo {
    pub fn simulation() -> Self {
        Self {
            label: "Simulation".to_string(),
            description: "Simulated Serial Stream (Test Mode)".to_string(),
            vid: None,
            pid: None,
            port_index: None,
            alias: None,
        }
    }

    pub fn fallback() -> Self {
        Self {
            label: "Connected".to_string(),
            description: "Connected Serial Port".to_string(),
            vid: None,
            pid: None,
            port_index: None,
            alias: None,
        }
    }

    pub fn update_alias(&mut self, new_alias: Option<String>) {
        self.alias = new_alias.filter(|a| !a.trim().is_empty());
        let base = identify_device(self.vid, self.pid);
        let base_label = base.label;

        if let Some(ref a) = self.alias {
            if let Some(idx) = self.port_index {
                self.label = format!("{a} [{base_label} #{idx}]");
            } else {
                self.label = format!("{a} [{base_label}]");
            }
        } else if let Some(idx) = self.port_index {
            self.label = format!("{base_label} #{idx}");
        } else {
            self.label = base_label;
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
                port_index: None,
                alias: None,
            };
        }

        if let Some(p) = pid {
            SerialDeviceInfo {
                label: format!("USB [{:04X}:{:04X}]", v, p),
                description: format!("USB Serial Device (VID: 0x{:04X}, PID: 0x{:04X})", v, p),
                vid,
                pid,
                port_index: None,
                alias: None,
            }
        } else {
            SerialDeviceInfo {
                label: format!("USB [{:04X}]", v),
                description: format!("USB Serial Device (VID: 0x{:04X})", v),
                vid,
                pid,
                port_index: None,
                alias: None,
            }
        }
    } else {
        SerialDeviceInfo::fallback()
    }
}

pub async fn get_port_info(port: &web_sys::SerialPort) -> SerialDeviceInfo {
    resolver::resolve_port_info(port).await
}
