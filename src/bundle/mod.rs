//! App bundle loading for curvy applications.
//!
//! An app bundle is a self-contained directory with the `.curvy` extension
//! that contains everything needed to run an application:
//!
//! ```text
//! my_app.curvy/
//! ├─ app.toml           # App configuration and metadata
//! ├─ skin/              # Visual assets
//! │  ├─ skin.json       # Skin definition
//! │  └─ images/         # Image assets
//! ├─ scripts/           # Lua scripts
//! │  └─ calculate.lua
//! └─ resources/         # Optional resources
//!    └─ icon.png
//! ```
//!
//! # app.toml Format
//!
//! ```toml
//! [app]
//! name = "My Application"
//! version = "1.0.0"
//!
//! [skin]
//! path = "skin/skin.json"
//!
//! [fonts]
//! default = "skin/fonts/font.ttf"
//! size = 16.0
//!
//! [actions]
//! calculate = "scripts/calculate.lua"
//! reset = "scripts/reset.lua"
//! ```

mod loader;

pub use loader::{AppBundle, BundleError};
