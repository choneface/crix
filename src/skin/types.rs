use std::collections::HashMap;
use std::path::PathBuf;

/// Skin metadata from [skin] section.
#[derive(Debug, Clone)]
pub struct SkinMeta {
    pub name: String,
    pub author: String,
    pub version: String,
}

/// Window configuration from [window] section.
#[derive(Debug, Clone)]
pub struct SkinWindow {
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
}

/// Drawing configuration for stateful widgets.
#[derive(Debug, Clone)]
pub struct PartDraw {
    pub normal: String,
    pub hover: String,
    pub pressed: String,
}

/// Hit testing configuration.
#[derive(Debug, Clone)]
pub struct PartHit {
    pub hit_type: HitType,
}

/// Hit region type.
#[derive(Debug, Clone)]
pub enum HitType {
    Rect,
}

/// Part type discriminator.
#[derive(Debug, Clone)]
pub enum PartType {
    Image { asset: String },
    Button,
}

/// A skin part definition from [[parts]] in TOML.
#[derive(Debug, Clone)]
pub struct SkinPart {
    pub id: String,
    pub part_type: PartType,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub z: i32,
    pub draw: Option<PartDraw>,
    pub hit: Option<PartHit>,
    pub action: Option<String>,
}

/// The root skin structure parsed from skin.toml.
#[derive(Debug, Clone)]
pub struct Skin {
    pub meta: SkinMeta,
    pub window: SkinWindow,
    pub assets: HashMap<String, PathBuf>,
    pub parts: Vec<SkinPart>,
}

/// Errors that can occur when loading a skin.
#[derive(Debug)]
pub enum SkinError {
    Io(std::io::Error),
    Json(serde_json::Error),
    AssetNotFound(String),
    MissingDrawSection(String),
    InvalidPartType(String),
    Image(image::ImageError),
}

impl std::fmt::Display for SkinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkinError::Io(e) => write!(f, "IO error: {}", e),
            SkinError::Json(e) => write!(f, "JSON parse error: {}", e),
            SkinError::AssetNotFound(key) => write!(f, "Asset not found: {}", key),
            SkinError::MissingDrawSection(id) => write!(f, "Missing 'draw' for button: {}", id),
            SkinError::InvalidPartType(t) => write!(f, "Invalid part type: {}", t),
            SkinError::Image(e) => write!(f, "Image error: {}", e),
        }
    }
}

impl std::error::Error for SkinError {}

impl From<std::io::Error> for SkinError {
    fn from(e: std::io::Error) -> Self {
        SkinError::Io(e)
    }
}

impl From<serde_json::Error> for SkinError {
    fn from(e: serde_json::Error) -> Self {
        SkinError::Json(e)
    }
}

impl From<image::ImageError> for SkinError {
    fn from(e: image::ImageError) -> Self {
        SkinError::Image(e)
    }
}
