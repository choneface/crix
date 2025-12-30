//! App configuration parsing.
//!
//! The app configuration lives in `app/app.toml` and maps action names to Lua script files.
//! This is intentionally separate from the skin pack - skins are purely aesthetic,
//! while app logic lives in the app directory.
//!
//! # Example app.toml
//!
//! ```toml
//! [app]
//! name = "Blend Calculator"
//! version = "1.0"
//!
//! [actions]
//! calculate_blend = "actions/calculate_blend.lua"
//! reset_form = "actions/reset_form.lua"
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// App metadata from [app] section.
#[derive(Debug, Clone, Deserialize)]
pub struct AppMeta {
    pub name: String,
    #[serde(default)]
    pub version: String,
}

/// Raw TOML structure.
#[derive(Debug, Deserialize)]
struct AppToml {
    app: AppMeta,
    #[serde(default)]
    actions: HashMap<String, String>,
}

/// Loaded app configuration with resolved script paths.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// App metadata.
    pub meta: AppMeta,
    /// Map of action name -> absolute script path.
    action_scripts: HashMap<String, PathBuf>,
    /// Base directory for the app (where app.toml lives).
    base_path: PathBuf,
}

/// Errors that can occur when loading app configuration.
#[derive(Debug)]
pub enum AppConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    ScriptNotFound { action: String, path: PathBuf },
}

impl std::fmt::Display for AppConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppConfigError::Io(e) => write!(f, "IO error: {}", e),
            AppConfigError::Toml(e) => write!(f, "TOML parse error: {}", e),
            AppConfigError::ScriptNotFound { action, path } => {
                write!(f, "Script for action '{}' not found: {:?}", action, path)
            }
        }
    }
}

impl std::error::Error for AppConfigError {}

impl From<std::io::Error> for AppConfigError {
    fn from(e: std::io::Error) -> Self {
        AppConfigError::Io(e)
    }
}

impl From<toml::de::Error> for AppConfigError {
    fn from(e: toml::de::Error) -> Self {
        AppConfigError::Toml(e)
    }
}

impl AppConfig {
    /// Load app configuration from an app.toml file.
    ///
    /// # Arguments
    /// * `path` - Path to the app.toml file
    ///
    /// # Returns
    /// The loaded configuration with all script paths resolved and validated.
    pub fn load(path: &Path) -> Result<Self, AppConfigError> {
        let content = fs::read_to_string(path)?;
        let toml: AppToml = toml::from_str(&content)?;

        let base_path = path.parent().unwrap_or(Path::new(".")).to_path_buf();

        // Resolve and validate script paths
        let mut action_scripts = HashMap::new();
        for (action_name, script_rel_path) in toml.actions {
            let script_path = base_path.join(&script_rel_path);
            if !script_path.exists() {
                return Err(AppConfigError::ScriptNotFound {
                    action: action_name,
                    path: script_path,
                });
            }
            action_scripts.insert(action_name, script_path);
        }

        Ok(Self {
            meta: toml.app,
            action_scripts,
            base_path,
        })
    }

    /// Get the script path for an action.
    ///
    /// Returns None if the action is not defined in the configuration.
    pub fn get_script(&self, action_name: &str) -> Option<&Path> {
        self.action_scripts.get(action_name).map(|p| p.as_path())
    }

    /// Get the base path of the app directory.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Check if an action is defined.
    pub fn has_action(&self, action_name: &str) -> bool {
        self.action_scripts.contains_key(action_name)
    }

    /// Get all registered action names.
    pub fn action_names(&self) -> impl Iterator<Item = &String> {
        self.action_scripts.keys()
    }
}
