use std::collections::HashMap;
use std::path::Path;

use image::{ImageReader, RgbImage};

use super::types::{Skin, SkinError, SkinWindow};

/// A skin with all assets loaded and ready to use.
pub struct LoadedSkin {
    pub skin: Skin,
    images: HashMap<String, RgbImage>,
}

impl LoadedSkin {
    /// Load a skin and all its assets from a TOML file path.
    pub fn load(path: &Path) -> Result<Self, SkinError> {
        let skin = Skin::load(path)?;

        let mut images = HashMap::new();

        // Load all image assets
        for (key, asset_path) in &skin.assets {
            let reader = ImageReader::open(asset_path)?;
            let img = reader.decode()?;
            let rgb = img.to_rgb8();
            images.insert(key.clone(), rgb);
        }

        Ok(Self { skin, images })
    }

    /// Get the window configuration from the skin.
    pub fn window(&self) -> &SkinWindow {
        &self.skin.window
    }

    /// Get the skin name.
    pub fn name(&self) -> &str {
        &self.skin.meta.name
    }

    /// Get an image by asset key.
    pub fn get_image(&self, key: &str) -> Option<&RgbImage> {
        self.images.get(key)
    }
}
