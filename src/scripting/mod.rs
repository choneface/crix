//! Scripting support for curvy applications.
//!
//! This module provides a scripting layer for defining application behavior
//! without recompiling the Rust code. Scripts live in the app directory
//! (NOT in skin packs) and can read/write values in the Store.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                         App Directory                           │
//! │  ┌──────────────┐  ┌─────────────────────────────────────────┐  │
//! │  │  app.toml    │  │  actions/                               │  │
//! │  │  [actions]   │──│    calculate_blend.lua                  │  │
//! │  │  calc = ...  │  │    reset_form.lua                       │  │
//! │  └──────────────┘  └─────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//!                                  │
//!                                  ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      LuaActionHandler                           │
//! │  ┌──────────────┐  ┌─────────────────────────────────────────┐  │
//! │  │  AppConfig   │  │  Lua VM (per-execution)                 │  │
//! │  │  action map  │──│  - app.get(key)                         │  │
//! │  └──────────────┘  │  - app.set(key, value)                  │  │
//! │                    │  - app.log(message)                     │  │
//! │                    └─────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//!                                  │
//!                                  ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                           Store                                 │
//! │  ┌──────────────────────────────────────────────────────────┐   │
//! │  │  inputs.current_ethanol_pct = "10"                       │   │
//! │  │  inputs.target_ethanol_pct = "30"                        │   │
//! │  │  outputs.e85_to_add_liters = "5.45"                      │   │
//! │  └──────────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Separation of Concerns
//!
//! - **Skins** are purely aesthetic (images, layouts, colors)
//! - **Scripts** define behavior (calculations, validations)
//! - **Store** is the bridge between UI and logic
//!
//! # Security Model
//!
//! Scripts are trusted (app-owned) but have a minimal API:
//! - NO filesystem access
//! - NO network access
//! - NO OS commands
//! - NO widget references
//! - Only Store read/write via `app.get()` / `app.set()`
//!
//! # Future Extensibility
//!
//! The design supports adding other scripting engines later:
//! - Implement `ActionHandler` for the new engine
//! - Use `ActionDispatcher` to chain handlers
//! - The Store + Action API remains stable

mod app_config;
mod lua_handler;

pub use app_config::{AppConfig, AppConfigError};
pub use lua_handler::{LuaActionHandler, LuaError};
