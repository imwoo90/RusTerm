//! Persistent command history manager backed by local storage.
//!
//! Stores previously executed commands in browser local storage, providing terminal-style
//! ArrowUp and ArrowDown recall across application sessions.

use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const HISTORY_KEY: &str = "cmd_history";
const MAX_HISTORY: usize = 50;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct CommandHistory {
    commands: Vec<String>,
}

impl CommandHistory {
    pub fn load() -> Self {
        #[cfg(target_arch = "wasm32")]
        if let Some(win) = window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(json)) = storage.get_item(HISTORY_KEY) {
                    if let Ok(history) = serde_json::from_str(&json) {
                        return history;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(win) = window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(json) = serde_json::to_string(self) {
                    let _ = storage.set_item(HISTORY_KEY, &json);
                }
            }
        }
    }

    pub fn add(&mut self, cmd: String) {
        if cmd.trim().is_empty() {
            return;
        }

        // Remove strictly logic: if same as last, don't add.
        if let Some(last) = self.commands.last() {
            if last == &cmd {
                return;
            }
        }

        self.commands.push(cmd);
        if self.commands.len() > MAX_HISTORY {
            self.commands.remove(0);
        }
        self.save();
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn get_at(&self, idx: usize) -> Option<&String> {
        self.commands.get(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_empty_initial() {
        let history = CommandHistory::default();
        assert_eq!(history.len(), 0);
        assert!(history.get_at(0).is_none());
    }

    #[test]
    fn test_history_add_and_retrieve() {
        let mut history = CommandHistory::default();
        history.add("AT".to_string());
        history.add("AT+GMR".to_string());

        assert_eq!(history.len(), 2);
        assert_eq!(history.get_at(0).unwrap(), "AT");
        assert_eq!(history.get_at(1).unwrap(), "AT+GMR");
    }

    #[test]
    fn test_history_ignores_empty_or_whitespace() {
        let mut history = CommandHistory::default();
        history.add("".to_string());
        history.add("   ".to_string());
        history.add("\t\n".to_string());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_history_dedup_consecutive() {
        let mut history = CommandHistory::default();
        history.add("PING".to_string());
        history.add("PING".to_string());
        history.add("PING".to_string());
        assert_eq!(history.len(), 1);

        history.add("PONG".to_string());
        history.add("PING".to_string());
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_history_fifo_capacity_cap() {
        let mut history = CommandHistory::default();
        for i in 0..60 {
            history.add(format!("CMD_{}", i));
        }

        assert_eq!(history.len(), MAX_HISTORY);
        // Oldest 10 evicted, should start at CMD_10
        assert_eq!(history.get_at(0).unwrap(), "CMD_10");
        assert_eq!(history.get_at(MAX_HISTORY - 1).unwrap(), "CMD_59");
    }

    #[test]
    fn test_history_serialization_roundtrip() {
        let mut history = CommandHistory::default();
        history.add("CMD_A".to_string());
        history.add("CMD_B".to_string());

        let json = serde_json::to_string(&history).expect("Serialize failed");
        let restored: CommandHistory = serde_json::from_str(&json).expect("Deserialize failed");

        assert_eq!(restored.len(), 2);
        assert_eq!(restored.get_at(0).unwrap(), "CMD_A");
        assert_eq!(restored.get_at(1).unwrap(), "CMD_B");
    }
}
