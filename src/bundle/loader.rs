//! App bundle loader implementation.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::skin::{LoadedSkin, SkinError};

/// App metadata from [app] section.
#[derive(Debug, Clone, Deserialize)]
pub struct AppMeta {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
}

/// Skin configuration from [skin] section.
#[derive(Debug, Clone, Deserialize)]
pub struct SkinConfig {
    /// Path to skin.json relative to bundle root.
    pub path: String,
}

/// Font configuration from [fonts] section.
#[derive(Debug, Clone, Deserialize)]
pub struct FontConfig {
    /// Path to default font file relative to bundle root.
    pub default: String,
    /// Default font size.
    #[serde(default = "default_font_size")]
    pub size: f32,
}

fn default_font_size() -> f32 {
    16.0
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            default: String::new(),
            size: 16.0,
        }
    }
}

/// Raw TOML structure for app.toml.
#[derive(Debug, Deserialize)]
struct AppToml {
    app: AppMeta,
    #[serde(default)]
    skin: Option<SkinConfig>,
    #[serde(default)]
    fonts: Option<FontConfig>,
    #[serde(default)]
    actions: HashMap<String, String>,
}

/// Errors that can occur when loading an app bundle.
#[derive(Debug)]
pub enum BundleError {
    /// Bundle directory not found.
    NotFound(PathBuf),
    /// app.toml not found in bundle.
    NoAppToml(PathBuf),
    /// IO error.
    Io(std::io::Error),
    /// TOML parse error.
    Toml(toml::de::Error),
    /// Skin loading error.
    Skin(SkinError),
    /// Font file not found.
    FontNotFound(PathBuf),
    /// Script file not found.
    ScriptNotFound { action: String, path: PathBuf },
    /// Skin not configured.
    NoSkin,
    /// Font not configured.
    NoFont,
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::NotFound(path) => write!(f, "Bundle not found: {:?}", path),
            BundleError::NoAppToml(path) => write!(f, "app.toml not found in: {:?}", path),
            BundleError::Io(e) => write!(f, "IO error: {}", e),
            BundleError::Toml(e) => write!(f, "TOML parse error: {}", e),
            BundleError::Skin(e) => write!(f, "Skin error: {}", e),
            BundleError::FontNotFound(path) => write!(f, "Font not found: {:?}", path),
            BundleError::ScriptNotFound { action, path } => {
                write!(f, "Script for action '{}' not found: {:?}", action, path)
            }
            BundleError::NoSkin => write!(f, "No skin configured in app.toml"),
            BundleError::NoFont => write!(f, "No font configured in app.toml"),
        }
    }
}

impl std::error::Error for BundleError {}

impl From<std::io::Error> for BundleError {
    fn from(e: std::io::Error) -> Self {
        BundleError::Io(e)
    }
}

impl From<toml::de::Error> for BundleError {
    fn from(e: toml::de::Error) -> Self {
        BundleError::Toml(e)
    }
}

impl From<SkinError> for BundleError {
    fn from(e: SkinError) -> Self {
        BundleError::Skin(e)
    }
}

/// A loaded app bundle with all resources resolved.
#[derive(Debug)]
pub struct AppBundle {
    /// Bundle root directory.
    root: PathBuf,
    /// App metadata.
    pub meta: AppMeta,
    /// Resolved skin path.
    skin_path: PathBuf,
    /// Resolved font path and size.
    font_path: PathBuf,
    pub font_size: f32,
    /// Action name -> script path mapping.
    action_scripts: HashMap<String, PathBuf>,
}

impl AppBundle {
    /// Load an app bundle from a directory.
    ///
    /// # Arguments
    /// * `path` - Path to the bundle directory (e.g., "my_app.curvy")
    ///
    /// # Returns
    /// A loaded AppBundle with all paths resolved and validated.
    pub fn load(path: &Path) -> Result<Self, BundleError> {
        // Verify bundle directory exists
        if !path.exists() {
            return Err(BundleError::NotFound(path.to_path_buf()));
        }

        let root = path.to_path_buf();

        // Load app.toml
        let app_toml_path = root.join("app.toml");
        if !app_toml_path.exists() {
            return Err(BundleError::NoAppToml(root.clone()));
        }

        let content = fs::read_to_string(&app_toml_path)?;
        let toml: AppToml = toml::from_str(&content)?;

        // Resolve skin path
        let skin_config = toml.skin.ok_or(BundleError::NoSkin)?;
        let skin_path = root.join(&skin_config.path);
        if !skin_path.exists() {
            return Err(BundleError::Skin(SkinError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Skin not found: {:?}", skin_path),
            ))));
        }

        // Resolve font path
        let font_config = toml.fonts.ok_or(BundleError::NoFont)?;
        let font_path = root.join(&font_config.default);
        if !font_path.exists() {
            return Err(BundleError::FontNotFound(font_path));
        }

        // Resolve action script paths
        let mut action_scripts = HashMap::new();
        for (action_name, script_rel_path) in toml.actions {
            let script_path = root.join(&script_rel_path);
            if !script_path.exists() {
                return Err(BundleError::ScriptNotFound {
                    action: action_name,
                    path: script_path,
                });
            }
            action_scripts.insert(action_name, script_path);
        }

        Ok(Self {
            root,
            meta: toml.app,
            skin_path,
            font_path,
            font_size: font_config.size,
            action_scripts,
        })
    }

    /// Get the bundle root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the path to the skin.json file.
    pub fn skin_path(&self) -> &Path {
        &self.skin_path
    }

    /// Get the path to the default font file.
    pub fn font_path(&self) -> &Path {
        &self.font_path
    }

    /// Get the script path for an action.
    pub fn get_script(&self, action_name: &str) -> Option<&Path> {
        self.action_scripts.get(action_name).map(|p| p.as_path())
    }

    /// Check if an action is defined.
    pub fn has_action(&self, action_name: &str) -> bool {
        self.action_scripts.contains_key(action_name)
    }

    /// Get all registered action names.
    pub fn action_names(&self) -> impl Iterator<Item = &String> {
        self.action_scripts.keys()
    }

    /// Load the skin from this bundle.
    pub fn load_skin(&self) -> Result<LoadedSkin, SkinError> {
        LoadedSkin::load(&self.skin_path)
    }

    /// Create an AppConfig compatible with the scripting module.
    /// This allows the LuaActionHandler to work with bundles.
    pub fn to_app_config(&self) -> AppConfigAdapter {
        AppConfigAdapter {
            meta_name: self.meta.name.clone(),
            meta_version: self.meta.version.clone(),
            action_scripts: self.action_scripts.clone(),
        }
    }
}

/// Adapter to make AppBundle work with the existing scripting infrastructure.
/// Implements the same interface as AppConfig but backed by bundle data.
#[derive(Debug, Clone)]
pub struct AppConfigAdapter {
    pub meta_name: String,
    pub meta_version: String,
    action_scripts: HashMap<String, PathBuf>,
}

impl AppConfigAdapter {
    /// Get the script path for an action.
    pub fn get_script(&self, action_name: &str) -> Option<&Path> {
        self.action_scripts.get(action_name).map(|p| p.as_path())
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
