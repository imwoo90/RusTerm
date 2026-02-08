use serde::{Deserialize, Serialize};
use web_sys::window;

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
        let id = js_sys::Date::now() as u64;
        self.items.push(MacroItem {
            id,
            label,
            command,
            is_hex,
            line_ending,
        });
        self.save();
    }

    pub fn remove(&mut self, id: u64) {
        self.items.retain(|item| item.id != id);
        self.save();
    }
}
