//! Vendor lookup tables and chipset metadata mappings for USB-to-UART serial bridges.
//!
//! Maps vendor IDs (VID) and product IDs (PID) to human-readable hardware identifiers and chip descriptions.

fn lookup_silabs(pid: Option<u16>) -> (&'static str, &'static str) {
    match pid {
        Some(0xEA60) => (
            "CP2102",
            "Silicon Labs CP2102/CP2109 USB-to-UART (VID: 0x10C4, PID: 0xEA60)",
        ),
        Some(0xEA70) => (
            "CP2105",
            "Silicon Labs CP2105 Dual USB-to-UART (VID: 0x10C4, PID: 0xEA70)",
        ),
        Some(0xEA71) => (
            "CP2108",
            "Silicon Labs CP2108 Quad USB-to-UART (VID: 0x10C4, PID: 0xEA71)",
        ),
        _ => ("CP210x", "Silicon Labs USB-to-UART Bridge"),
    }
}

fn lookup_wch(pid: Option<u16>) -> (&'static str, &'static str) {
    match pid {
        Some(0x7523) => (
            "CH340",
            "WCH CH340 USB-to-Serial (VID: 0x1A86, PID: 0x7523)",
        ),
        Some(0x5523) => (
            "CH341",
            "WCH CH341 USB-to-Serial (VID: 0x1A86, PID: 0x5523)",
        ),
        Some(0x7522) => (
            "CH340K",
            "WCH CH340K USB-to-Serial (VID: 0x1A86, PID: 0x7522)",
        ),
        Some(0x55D4) => (
            "CH9102",
            "WCH CH9102 USB-to-Serial (VID: 0x1A86, PID: 0x55D4)",
        ),
        Some(0x5512) => (
            "CH341A",
            "WCH CH341A USB-to-Serial/Parallel (VID: 0x1A86, PID: 0x5512)",
        ),
        _ => ("CH34x", "WCH USB-to-Serial Bridge"),
    }
}

fn lookup_ftdi(pid: Option<u16>) -> (&'static str, &'static str) {
    match pid {
        Some(0x6001) => (
            "FT232R",
            "FTDI FT232R USB-to-UART (VID: 0x0403, PID: 0x6001)",
        ),
        Some(0x6010) => (
            "FT2232H",
            "FTDI FT2232H Dual High-Speed USB-to-UART (VID: 0x0403, PID: 0x6010)",
        ),
        Some(0x6011) => (
            "FT4232H",
            "FTDI FT4232H Quad High-Speed USB-to-UART (VID: 0x0403, PID: 0x6011)",
        ),
        Some(0x6014) => (
            "FT232H",
            "FTDI FT232H High-Speed USB-to-UART (VID: 0x0403, PID: 0x6014)",
        ),
        Some(0x6015) => (
            "FT231X",
            "FTDI FT230X/FT231X USB-to-UART (VID: 0x0403, PID: 0x6015)",
        ),
        _ => ("FTDI", "FTDI USB Serial Device"),
    }
}

fn lookup_espressif(pid: Option<u16>) -> (&'static str, &'static str) {
    match pid {
        Some(0x1001) => (
            "ESP32-S3/C3",
            "Espressif USB JTAG/Serial debug unit (VID: 0x303A, PID: 0x1001)",
        ),
        Some(0x1002) | Some(0x0002) => (
            "ESP32-S2",
            "Espressif ESP32-S2 USB CDC (VID: 0x303A, PID: 0x1002)",
        ),
        Some(0x1000) => (
            "ESP32-C3",
            "Espressif ESP32-C3 USB CDC (VID: 0x303A, PID: 0x1000)",
        ),
        _ => ("ESP32", "Espressif USB Serial Device"),
    }
}

fn lookup_raspberry_pi(pid: Option<u16>) -> (&'static str, &'static str) {
    match pid {
        Some(0x000A) => (
            "RP2040 Pico",
            "Raspberry Pi Pico USB Serial (VID: 0x2E8A, PID: 0x000A)",
        ),
        Some(0x0005) => (
            "RP2040 Probe",
            "Raspberry Pi Picoprobe CMSIS-DAP (VID: 0x2E8A, PID: 0x0005)",
        ),
        _ => ("RP2040", "Raspberry Pi USB Serial Device"),
    }
}

fn lookup_arduino(vid: u16, pid: Option<u16>) -> (&'static str, &'static str) {
    if vid == 0x2341 {
        match pid {
            Some(0x0043) | Some(0x0001) => {
                ("Arduino Uno", "Arduino Uno R3 (VID: 0x2341, PID: 0x0043)")
            }
            Some(0x0042) | Some(0x0010) => {
                ("Arduino Mega", "Arduino Mega 2560 (VID: 0x2341, PID: 0x0042)")
            }
            Some(0x8036) => (
                "Arduino Leonardo",
                "Arduino Leonardo (VID: 0x2341, PID: 0x8036)",
            ),
            Some(0x1002) => (
                "Arduino UNO R4",
                "Arduino UNO R4 WiFi/Minima (VID: 0x2341, PID: 0x1002)",
            ),
            _ => ("Arduino", "Arduino USB Serial Device"),
        }
    } else {
        ("Arduino", "Arduino Serial Device")
    }
}

fn lookup_stmicro(pid: Option<u16>) -> (&'static str, &'static str) {
    match pid {
        Some(0x3748) | Some(0x374B) | Some(0x3752) | Some(0x3754) => {
            ("ST-Link", "STMicroelectronics ST-LINK VCP (VID: 0x0483)")
        }
        Some(0x5740) => (
            "STM32 VCP",
            "STMicroelectronics STM32 Virtual COM Port (VID: 0x0483, PID: 0x5740)",
        ),
        _ => ("STM32", "STMicroelectronics USB Device"),
    }
}

pub fn lookup_known_vendor(vid: u16, pid: Option<u16>) -> Option<(&'static str, &'static str)> {
    match vid {
        0x10C4 => Some(lookup_silabs(pid)),
        0x1A86 => Some(lookup_wch(pid)),
        0x0403 => Some(lookup_ftdi(pid)),
        0x303A => Some(lookup_espressif(pid)),
        0x2E8A => Some(lookup_raspberry_pi(pid)),
        0x2341 | 0x2A03 => Some(lookup_arduino(vid, pid)),
        0x0483 => Some(lookup_stmicro(pid)),
        0x067B => Some(("PL2303", "Prolific PL2303 USB-to-Serial (VID: 0x067B)")),
        0x0D28 => Some(("DAPLink", "ARM DAPLink / CMSIS-DAP (VID: 0x0D28)")),
        0x1366 => Some(("J-Link", "SEGGER J-Link CDC Serial (VID: 0x1366)")),
        _ => None,
    }
}
