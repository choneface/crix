use std::path::Path;

use curvy::{
    run, init_font, Action, ActionDispatcher, App, AppConfig, KeyCode,
    LoadedSkin, LuaActionHandler, RunConfig, Services, SkinBuilder, StaticText,
    Store, TextInput, UiTree, View, WidgetEvent,
};
use winit::event::WindowEvent;
use winit::keyboard::{Key, NamedKey};

struct SkinApp {
    tree: UiTree,
    title: String,
    store: Store,
    dispatcher: ActionDispatcher,
    services: Services,
}

impl SkinApp {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load skin from directory
        let skin = LoadedSkin::load(Path::new("skins/classic/skin.json"))?;
        let title = skin.name().to_string();

        // Build UI tree from skin
        let (tree, _window_config) = SkinBuilder::build(&skin)?;

        // Set up the store and dispatcher
        let store = Store::new();
        let mut dispatcher = ActionDispatcher::new();

        // Load app configuration and create Lua action handler
        let app_config = AppConfig::load(Path::new("app/app.toml"))?;
        println!("Loaded app config: {}", app_config.meta.name);
        for action_name in app_config.action_names() {
            println!("  Registered action: {}", action_name);
        }
        let lua_handler = LuaActionHandler::new(app_config);
        dispatcher.add_handler(lua_handler);

        let services = Services::new();

        Ok(Self {
            tree,
            title,
            store,
            dispatcher,
            services,
        })
    }

    /// Sync text inputs to store (write dirty values).
    fn sync_inputs_to_store(&mut self) {
        let node_ids: Vec<_> = self.tree.iter_node_ids().collect();

        for id in node_ids {
            if let Some(node) = self.tree.get_mut(id) {
                if let Some(text_input) = node.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
                    if text_input.is_dirty() {
                        if let Some(binding) = text_input.binding() {
                            let text = text_input.text().to_string();
                            self.store.set(binding.to_string(), text);
                        }
                        text_input.clear_dirty();
                    }
                }
            }
        }
    }

    /// Sync store values to static text widgets (update displays).
    fn sync_store_to_outputs(&mut self) {
        let node_ids: Vec<_> = self.tree.iter_node_ids().collect();

        for id in node_ids {
            if let Some(node) = self.tree.get_mut(id) {
                if let Some(static_text) = node.widget_mut().as_any_mut().downcast_mut::<StaticText>() {
                    if let Some(binding) = static_text.binding() {
                        let value = self.store.get_string(binding);
                        if !value.is_empty() && value != static_text.content() {
                            static_text.set_content(value);
                        }
                    }
                }
            }
        }
    }

    /// Dispatch an action by name.
    fn dispatch_action(&mut self, name: &str) {
        let action = Action::new(name);
        if let Err(e) = self.dispatcher.dispatch(&action, &mut self.store, &self.services) {
            eprintln!("Action error: {}", e);
        }
    }

    /// Get the action for a clicked widget (if it's a button).
    fn get_button_action(&self, node_id: curvy::NodeId) -> Option<String> {
        if let Some(node) = self.tree.get(node_id) {
            // Try to get the action from a SkinButton
            if let Some(button) = node.widget().as_any().downcast_ref::<curvy::skin::widgets::SkinButton>() {
                return button.action().map(|s| s.to_string());
            }
        }
        None
    }
}

impl App for SkinApp {
    fn view(&self) -> &dyn View {
        &self.tree
    }

    fn on_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let hit = self.tree.hit_test(position.x as i32, position.y as i32);
                self.tree.set_hovered(hit);
                true
            }
            WindowEvent::MouseInput { state, .. } => {
                match state {
                    winit::event::ElementState::Pressed => {
                        // Set pressed state
                        if let Some(hovered) = self.tree.hovered() {
                            self.tree.set_pressed(Some(hovered));

                            // Focus the clicked widget (for text inputs)
                            let old_focused = self.tree.focused();
                            if old_focused != Some(hovered) {
                                // Notify old focused widget of focus loss
                                if let Some(old_id) = old_focused {
                                    if let Some(node) = self.tree.get_mut(old_id) {
                                        node.widget_mut().on_event(&WidgetEvent::FocusLost);
                                    }
                                }
                                // Set new focus
                                self.tree.set_focused(Some(hovered));
                                // Notify new widget of focus gain
                                if let Some(node) = self.tree.get_mut(hovered) {
                                    node.widget_mut().on_event(&WidgetEvent::FocusGained);
                                }
                            }
                        } else {
                            // Clicked outside any widget, clear focus
                            if let Some(old_id) = self.tree.focused() {
                                if let Some(node) = self.tree.get_mut(old_id) {
                                    node.widget_mut().on_event(&WidgetEvent::FocusLost);
                                }
                            }
                            self.tree.set_focused(None);
                        }
                    }
                    winit::event::ElementState::Released => {
                        if let Some(pressed_id) = self.tree.pressed() {
                            // Check if we're still hovering the pressed widget
                            if self.tree.hovered() == Some(pressed_id) {
                                // Get action before mutably borrowing tree
                                let action = self.get_button_action(pressed_id);

                                // Send click event to widget
                                if let Some(node) = self.tree.get_mut(pressed_id) {
                                    node.widget_mut().on_event(&WidgetEvent::Click);
                                }

                                // Dispatch action if this was a button
                                if let Some(action_name) = action {
                                    // Sync inputs first
                                    self.sync_inputs_to_store();
                                    // Dispatch the action
                                    self.dispatch_action(&action_name);
                                    // Sync outputs after action
                                    self.sync_store_to_outputs();
                                }
                            }
                        }
                        self.tree.set_pressed(None);
                    }
                }
                true
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !event.state.is_pressed() {
                    return false;
                }

                // Route keyboard events to focused widget
                if let Some(focused_id) = self.tree.focused() {
                    let widget_event = match &event.logical_key {
                        Key::Named(NamedKey::Backspace) => {
                            Some(WidgetEvent::KeyDown { key: KeyCode::Backspace })
                        }
                        Key::Named(NamedKey::Delete) => {
                            Some(WidgetEvent::KeyDown { key: KeyCode::Delete })
                        }
                        Key::Named(NamedKey::ArrowLeft) => {
                            Some(WidgetEvent::KeyDown { key: KeyCode::Left })
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            Some(WidgetEvent::KeyDown { key: KeyCode::Right })
                        }
                        Key::Named(NamedKey::Home) => {
                            Some(WidgetEvent::KeyDown { key: KeyCode::Home })
                        }
                        Key::Named(NamedKey::End) => {
                            Some(WidgetEvent::KeyDown { key: KeyCode::End })
                        }
                        Key::Named(NamedKey::Enter) => {
                            Some(WidgetEvent::KeyDown { key: KeyCode::Enter })
                        }
                        Key::Character(s) => {
                            // Only handle single ASCII characters
                            if s.len() == 1 {
                                let c = s.chars().next().unwrap();
                                if c as u32 >= 32 && c as u32 <= 126 {
                                    Some(WidgetEvent::CharInput { c })
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        Key::Named(NamedKey::Space) => {
                            Some(WidgetEvent::CharInput { c: ' ' })
                        }
                        _ => None,
                    };

                    if let Some(widget_event) = widget_event {
                        if let Some(node) = self.tree.get_mut(focused_id) {
                            node.widget_mut().on_event(&widget_event);
                        }
                        // Sync after input
                        self.sync_inputs_to_store();
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

fn main() {
    // Initialize font system - requires a TTF file
    init_font(Path::new("fonts/font.ttf"), 16.0)
        .expect("Failed to load font. Please place a TTF file at fonts/font.ttf");

    let app = SkinApp::new().expect("Failed to load skin or app config");
    let config = RunConfig::default().with_title(&app.title);

    run(app, config);
}
