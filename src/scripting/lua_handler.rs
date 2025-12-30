//! Lua-backed ActionHandler implementation.
//!
//! This module provides a Lua scripting backend for handling actions.
//! Scripts are loaded from the app directory (NOT from skin packs) and
//! can read/write values in the Store.
//!
//! # Lua API
//!
//! Scripts have access to a single global table `app` with the following functions:
//!
//! - `app.get(key)` - Get a value from the Store. Returns string, number, boolean, or nil.
//! - `app.set(key, value)` - Set a value in the Store. Value can be string, number, or boolean.
//! - `app.log(message)` - Log a message for debugging purposes.
//!
//! The `app.payload` table contains any payload data passed with the action (optional).
//!
//! # Security Model
//!
//! Scripts are considered trusted (app-owned) but the API is intentionally minimal:
//! - NO filesystem access
//! - NO network access
//! - NO OS commands
//! - NO widget/node references
//! - Only Store read/write is permitted
//!
//! # Example Script
//!
//! ```lua
//! -- calculate_blend.lua
//! local current = tonumber(app.get("inputs.current_ethanol_pct")) or 0
//! local target = tonumber(app.get("inputs.target_ethanol_pct")) or 0
//! local fuel = tonumber(app.get("inputs.current_fuel_liters")) or 0
//!
//! local E85_PCT = 85
//!
//! if target >= E85_PCT then
//!     app.set("outputs.e85_to_add_liters", "N/A")
//! elseif target <= current then
//!     app.set("outputs.e85_to_add_liters", "0.00")
//! else
//!     local result = (target - current) * fuel / (E85_PCT - target)
//!     app.set("outputs.e85_to_add_liters", string.format("%.2f", result))
//! end
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use mlua::{Lua, Value as LuaValue};

use crate::core::{Action, ActionError, ActionHandler, Services, Store, Value};

use super::app_config::AppConfig;

/// Errors that can occur during Lua script execution.
#[derive(Debug)]
pub enum LuaError {
    /// IO error reading script file.
    Io(std::io::Error),
    /// Lua runtime error.
    Runtime(String),
    /// Script not found for action.
    ScriptNotFound(String),
}

impl std::fmt::Display for LuaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaError::Io(e) => write!(f, "Script IO error: {}", e),
            LuaError::Runtime(msg) => write!(f, "Lua runtime error: {}", msg),
            LuaError::ScriptNotFound(action) => write!(f, "No script for action: {}", action),
        }
    }
}

impl std::error::Error for LuaError {}

impl From<mlua::Error> for LuaError {
    fn from(e: mlua::Error) -> Self {
        LuaError::Runtime(e.to_string())
    }
}

/// A Lua-backed action handler.
///
/// Executes Lua scripts in response to actions, allowing scripts to
/// read and write Store values.
pub struct LuaActionHandler {
    /// Action name -> script path mappings.
    action_scripts: HashMap<String, PathBuf>,
}

impl LuaActionHandler {
    /// Create a new Lua action handler from an app configuration.
    ///
    /// # Arguments
    /// * `config` - The app configuration containing action -> script mappings
    ///
    /// # Returns
    /// A new LuaActionHandler ready to process actions.
    pub fn new(config: AppConfig) -> Self {
        let mut action_scripts = HashMap::new();
        for action_name in config.action_names() {
            if let Some(path) = config.get_script(action_name) {
                action_scripts.insert(action_name.clone(), path.to_path_buf());
            }
        }
        Self { action_scripts }
    }

    /// Create a new Lua action handler from a HashMap of action -> script path.
    ///
    /// This is useful when loading from an app bundle.
    pub fn from_scripts(action_scripts: HashMap<String, PathBuf>) -> Self {
        Self { action_scripts }
    }

    /// Get the script path for an action.
    pub fn get_script(&self, action_name: &str) -> Option<&Path> {
        self.action_scripts.get(action_name).map(|p| p.as_path())
    }

    /// Get all registered action names.
    pub fn action_names(&self) -> impl Iterator<Item = &String> {
        self.action_scripts.keys()
    }

    /// Execute a Lua script with access to the Store.
    ///
    /// Creates a fresh Lua VM for each script execution to ensure isolation.
    /// Sets up the `app` global table with get/set/log functions.
    fn execute_script(
        &self,
        script_path: &Path,
        action: &Action,
        store: &mut Store,
    ) -> Result<(), LuaError> {
        // Read the script
        let script_content = fs::read_to_string(script_path).map_err(LuaError::Io)?;

        // Create a fresh Lua VM for this script
        let lua = Lua::new();

        // Remove dangerous globals for safety (even though scripts are trusted)
        lua.globals().set("os", LuaValue::Nil)?;
        lua.globals().set("io", LuaValue::Nil)?;
        lua.globals().set("debug", LuaValue::Nil)?;
        lua.globals().set("loadfile", LuaValue::Nil)?;
        lua.globals().set("dofile", LuaValue::Nil)?;
        lua.globals().set("load", LuaValue::Nil)?;

        // We'll use a two-phase approach:
        // 1. Copy store values into Lua tables
        // 2. Run the script
        // 3. Copy modified values back to the store

        // Create a table to hold store values (for reading)
        let store_data = lua.create_table()?;
        for key in store.keys() {
            if let Some(value) = store.get(key) {
                match value {
                    Value::String(s) => store_data.set(key.clone(), s.clone())?,
                    Value::Number(n) => store_data.set(key.clone(), *n)?,
                    Value::Bool(b) => store_data.set(key.clone(), *b)?,
                    Value::Null => store_data.set(key.clone(), LuaValue::Nil)?,
                }
            }
        }

        // Create a table to collect outputs
        let output_data = lua.create_table()?;

        // Create a table to collect log messages
        let log_messages = lua.create_table()?;

        // Create the app table with get/set/log functions
        let app_table = lua.create_table()?;

        // app.get(key) - read from store_data
        let store_data_ref = store_data.clone();
        let get_fn = lua.create_function(move |_lua, key: String| {
            let value: LuaValue = store_data_ref.get(key)?;
            Ok(value)
        })?;
        app_table.set("get", get_fn)?;

        // app.set(key, value) - write to output_data
        let output_data_ref = output_data.clone();
        let set_fn = lua.create_function(move |_, (key, value): (String, LuaValue)| {
            output_data_ref.set(key, value)?;
            Ok(())
        })?;
        app_table.set("set", set_fn)?;

        // app.log(message) - collect log messages
        let log_messages_ref = log_messages.clone();
        let log_fn = lua.create_function(move |_, message: String| {
            let len: i64 = log_messages_ref.len()? + 1;
            log_messages_ref.set(len, message)?;
            Ok(())
        })?;
        app_table.set("log", log_fn)?;

        // app.payload - action payload table
        let payload_table = lua.create_table()?;
        for (key, value) in &action.payload {
            match value {
                Value::String(s) => payload_table.set(key.clone(), s.clone())?,
                Value::Number(n) => payload_table.set(key.clone(), *n)?,
                Value::Bool(b) => payload_table.set(key.clone(), *b)?,
                Value::Null => payload_table.set(key.clone(), LuaValue::Nil)?,
            }
        }
        app_table.set("payload", payload_table)?;

        // Set the app global
        lua.globals().set("app", app_table)?;

        // Execute the script
        lua.load(&script_content)
            .set_name(script_path.to_string_lossy())
            .exec()?;

        // Copy output values back to the store
        for pair in output_data.pairs::<String, LuaValue>() {
            let (key, value) = pair?;
            match value {
                LuaValue::String(s) => store.set(key, s.to_str()?.to_string()),
                LuaValue::Number(n) => store.set(key, n),
                LuaValue::Integer(i) => store.set(key, i as f64),
                LuaValue::Boolean(b) => store.set(key, b),
                LuaValue::Nil => store.set(key, Value::Null),
                _ => {
                    // Ignore other types (tables, functions, etc.)
                    eprintln!("Warning: Ignoring non-primitive value for key '{}'", key);
                }
            }
        }

        // Print any log messages
        for i in 1..=log_messages.len()? {
            let msg: String = log_messages.get(i)?;
            println!("[Lua] {}", msg);
        }

        Ok(())
    }
}

impl ActionHandler for LuaActionHandler {
    fn handle(
        &mut self,
        action: &Action,
        store: &mut Store,
        _services: &Services,
    ) -> Result<bool, ActionError> {
        // Look up the script for this action
        let script_path = match self.get_script(&action.name) {
            Some(path) => path.to_path_buf(),
            None => return Ok(false), // Action not handled by Lua
        };

        // Execute the script
        match self.execute_script(&script_path, action, store) {
            Ok(()) => Ok(true),
            Err(e) => {
                // Log the error
                eprintln!("Lua script error for action '{}': {}", action.name, e);

                // Store error message for UI feedback
                store.set(
                    format!("errors.action.{}", action.name),
                    format!("Script error: {}", e),
                );

                // Don't crash - return handled but with error logged
                Ok(true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_basic_execution() {
        // This would require a test app config and script
        // For now, just verify the types compile
    }
}
