//! Unit tests for USB-to-UART serial chipset identification and metadata resolution.
//!
//! Validates vendor-specific lookup tables, generic USB fallbacks, and simulation profiles.

use super::*;

#[test]
fn test_identify_silabs() {
    let dev = identify_device(Some(0x10C4), Some(0xEA60));
    assert_eq!(dev.label, "CP2102");
    assert!(dev.description.contains("Silicon Labs"));

    let dev2 = identify_device(Some(0x10C4), Some(0xEA70));
    assert_eq!(dev2.label, "CP2105");

    let dev3 = identify_device(Some(0x10C4), Some(0x9999));
    assert_eq!(dev3.label, "CP210x");
}

#[test]
fn test_identify_wch() {
    let dev = identify_device(Some(0x1A86), Some(0x7523));
    assert_eq!(dev.label, "CH340");
    assert!(dev.description.contains("WCH"));

    let dev2 = identify_device(Some(0x1A86), Some(0x55D4));
    assert_eq!(dev2.label, "CH9102");
}

#[test]
fn test_identify_ftdi() {
    let dev = identify_device(Some(0x0403), Some(0x6001));
    assert_eq!(dev.label, "FT232R");
    assert!(dev.description.contains("FTDI"));

    let dev2 = identify_device(Some(0x0403), Some(0x6010));
    assert_eq!(dev2.label, "FT2232H");
}

#[test]
fn test_identify_espressif() {
    let dev = identify_device(Some(0x303A), Some(0x1001));
    assert_eq!(dev.label, "ESP32-S3/C3");

    let dev2 = identify_device(Some(0x303A), Some(0x1002));
    assert_eq!(dev2.label, "ESP32-S2");
}

#[test]
fn test_identify_raspberry_pi() {
    let dev = identify_device(Some(0x2E8A), Some(0x000A));
    assert_eq!(dev.label, "RP2040 Pico");
}

#[test]
fn test_identify_arduino() {
    let dev = identify_device(Some(0x2341), Some(0x0043));
    assert_eq!(dev.label, "Arduino Uno");
}

#[test]
fn test_identify_stmicro() {
    let dev = identify_device(Some(0x0483), Some(0x3748));
    assert_eq!(dev.label, "ST-Link");

    let dev2 = identify_device(Some(0x0483), Some(0x5740));
    assert_eq!(dev2.label, "STM32 VCP");
}

#[test]
fn test_identify_unknown_usb() {
    let dev = identify_device(Some(0x1234), Some(0x5678));
    assert_eq!(dev.label, "USB [1234:5678]");
    assert!(dev.description.contains("0x1234"));

    let dev2 = identify_device(Some(0x1234), None);
    assert_eq!(dev2.label, "USB [1234]");
}

#[test]
fn test_identify_fallback_and_simulation() {
    let fallback = identify_device(None, None);
    assert_eq!(fallback.label, "Connected");

    let sim = SerialDeviceInfo::simulation();
    assert_eq!(sim.label, "Simulation");
}

#[test]
fn test_multi_device_disambiguation() {
    // Single device - no number suffix needed
    let dev1 = build_device_info(Some(0x1A86), Some(0x7523), Some(1), 1, None);
    assert_eq!(dev1.label, "CH340");

    // Multiple identical devices - port numbers appended
    let dev_multi1 = build_device_info(Some(0x1A86), Some(0x7523), Some(1), 2, None);
    assert_eq!(dev_multi1.label, "CH340 #1");
    assert!(dev_multi1.description.contains("Port #1"));

    let dev_multi2 = build_device_info(Some(0x1A86), Some(0x7523), Some(2), 2, None);
    assert_eq!(dev_multi2.label, "CH340 #2");
    assert!(dev_multi2.description.contains("Port #2"));
}

#[test]
fn test_device_alias_handling() {
    // With alias set on a single port
    let dev = build_device_info(
        Some(0x10C4),
        Some(0xEA60),
        Some(1),
        1,
        Some("센서보드".to_string()),
    );
    assert_eq!(dev.label, "센서보드 [CP2102 #1]");
    assert!(dev.description.contains("센서보드"));

    // Dynamic alias update
    let mut dev2 = build_device_info(Some(0x10C4), Some(0xEA60), Some(2), 2, None);
    assert_eq!(dev2.label, "CP2102 #2");

    dev2.update_alias(Some("모터제어기".to_string()));
    assert_eq!(dev2.label, "모터제어기 [CP2102 #2]");

    dev2.update_alias(None);
    assert_eq!(dev2.label, "CP2102 #2");
}
