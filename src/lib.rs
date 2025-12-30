pub mod core;
pub mod graphics;
pub mod platform;
pub mod scripting;
pub mod skin;
pub mod widgets;

// Re-export commonly used types at the crate root
pub use core::{
    Action, ActionDispatcher, ActionError, ActionHandler, App, AppRunner, KeyCode, Node, NodeId,
    Rect, Services, Store, UiTree, Value, View, Widget, WidgetEvent, WidgetState,
};
pub use graphics::{Canvas, Image, init_font, FontError};
pub use platform::{run, RunConfig};
pub use scripting::{AppConfig, AppConfigError, LuaActionHandler, LuaError};
pub use skin::{LoadedSkin, SkinBuilder, SkinError, SkinWindow, StaticText, TextAlign, TextInput, VerticalAlign};
pub use widgets::{Button, Container, ImageWidget};
