//! # Serial Hardware Simulation Generator
//!
//! ## Overview
//! Synthesizes mock serial log streams with sensor data, ANSI color codes, and errors to verify end-to-end performance and filtering without physical serial devices.
//!
//! ## Search Tags
//! #simulation, #mock-serial, #testing, #stream-generator, #synthetic-data

/// Generates a single simulated log chunk with randomized sensor data or warnings
pub fn generate_simulated_chunk() -> js_sys::Uint8Array {
    let rnd = js_sys::Math::random();
    let mut bytes = Vec::new();

    if rnd < 0.05 {
        // Simulate garbage / corrupted data (invalid UTF-8)
        bytes.extend_from_slice(&[0xFF, 0xC0, 0xFE, 0x80, 0x12, 0x34]);
    } else if rnd < 0.15 {
        bytes.extend_from_slice(
            format!("Error: System overheat at {:.1}°C\n", 80.0 + rnd * 20.0).as_bytes(),
        );
    } else if rnd < 0.35 {
        bytes.extend_from_slice(
            format!("Warning: Voltage fluctuation detected: {:.2}V\n", 3.0 + rnd).as_bytes(),
        );
    } else {
        bytes.extend_from_slice(
            format!(
                "Info: Sensor reading: A={:.2}, B={:.2}, C={:.2}\n",
                rnd * 100.0,
                rnd * 50.0,
                rnd * 10.0
            )
            .as_bytes(),
        );
    }

    js_sys::Uint8Array::from(bytes.as_slice())
}
