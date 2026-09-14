//! # Quick Command Macro Storage Manager
//!
//! ## Overview
//! Manages persistent storage and serialization for user-defined macro buttons, saving command strings, labels, and shortcuts to browser local storage.
//!
//! ## Search Tags
//! #macros, #macro-storage, #local-storage, #command-shortcuts, #persistence

use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const MACRO_KEY: &str = "cmd_macros";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MacroItem {
    pub id: u64,
    pub label: String,
    pub command: String,
    #[serde(default)]
    pub is_hex: bool,
    #[serde(default)]
    pub line_ending: crate::state::LineEnding,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MacroStorage {
    items: Vec<MacroItem>,
}

impl MacroStorage {
    pub fn load() -> Self {
        #[cfg(target_arch = "wasm32")]
        if let Some(win) = window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(json)) = storage.get_item(MACRO_KEY) {
                    if let Ok(macros) = serde_json::from_str(&json) {
                        return macros;
                    }
                }
            }
        }
        // Default: Empty
        Self { items: Vec::new() }
    }

    pub fn save(&self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(win) = window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(json) = serde_json::to_string(self) {
                    let _ = storage.set_item(MACRO_KEY, &json);
                }
            }
        }
    }

    pub fn get_items(&self) -> Vec<MacroItem> {
        self.items.clone()
    }

    pub fn add(
        &mut self,
        label: String,
        command: String,
        is_hex: bool,
        line_ending: crate::state::LineEnding,
    ) {
        #[cfg(target_arch = "wasm32")]
        let id = js_sys::Date::now() as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.items.push(MacroItem {
            id,
            label,
            command,
            is_hex,
            line_ending,
        });
        self.save();
    }

    pub fn get(&self, id: u64) -> Option<MacroItem> {
        self.items.iter().find(|i| i.id == id).cloned()
    }

    pub fn update(
        &mut self,
        id: u64,
        label: String,
        command: String,
        is_hex: bool,
        line_ending: crate::state::LineEnding,
    ) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.label = label;
            item.command = command;
            item.is_hex = is_hex;
            item.line_ending = line_ending;
            self.save();
        }
    }

    pub fn remove(&mut self, id: u64) {
        self.items.retain(|item| item.id != id);
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::LineEnding;

    #[test]
    fn test_macro_empty_initial() {
        let storage = MacroStorage::default();
        assert_eq!(storage.get_items().len(), 0);
        assert!(storage.get(1).is_none());
    }

    #[test]
    fn test_macro_add_and_retrieve() {
        let mut storage = MacroStorage::default();
        storage.add("Status".to_string(), "AT+STATUS".to_string(), false, LineEnding::NLCR);

        let items = storage.get_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Status");
        assert_eq!(items[0].command, "AT+STATUS");
        assert_eq!(items[0].is_hex, false);
        assert_eq!(items[0].line_ending, LineEnding::NLCR);

        let retrieved = storage.get(items[0].id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().label, "Status");
    }

    #[test]
    fn test_macro_update() {
        let mut storage = MacroStorage::default();
        storage.add("Old".to_string(), "11 22".to_string(), true, LineEnding::None);
        let id = storage.get_items()[0].id;

        storage.update(id, "New".to_string(), "33 44".to_string(), true, LineEnding::NL);
        let updated = storage.get(id).expect("Item should exist");
        assert_eq!(updated.label, "New");
        assert_eq!(updated.command, "33 44");
        assert_eq!(updated.is_hex, true);
        assert_eq!(updated.line_ending, LineEnding::NL);
    }

    #[test]
    fn test_macro_remove() {
        let mut storage = MacroStorage::default();
        storage.add("First".to_string(), "CMD1".to_string(), false, LineEnding::CR);
        let id = storage.get_items()[0].id;

        storage.remove(id);
        assert_eq!(storage.get_items().len(), 0);
        assert!(storage.get(id).is_none());
    }

    #[test]
    fn test_macro_serialization_roundtrip() {
        let mut storage = MacroStorage::default();
        storage.add("M1".to_string(), "C1".to_string(), false, LineEnding::NLCR);
        storage.add("M2".to_string(), "AA BB".to_string(), true, LineEnding::None);

        let json = serde_json::to_string(&storage).expect("Serialize failed");
        let restored: MacroStorage = serde_json::from_str(&json).expect("Deserialize failed");

        let items = restored.get_items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].label, "M1");
        assert_eq!(items[1].label, "M2");
        assert_eq!(items[1].is_hex, true);
    }
}
