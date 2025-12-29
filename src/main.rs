use std::path::Path;

use curvy::{run, init_font, App, KeyCode, LoadedSkin, RunConfig, SkinBuilder, UiTree, View, WidgetEvent};
use winit::event::WindowEvent;
use winit::keyboard::{Key, NamedKey};

struct SkinApp {
    tree: UiTree,
    title: String,
}

impl SkinApp {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load skin from directory
        let skin = LoadedSkin::load(Path::new("skins/classic/skin.json"))?;
        let title = skin.name().to_string();

        // Build UI tree from skin
        let (tree, _window_config) = SkinBuilder::build(&skin)?;

        Ok(Self { tree, title })
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
                                if let Some(node) = self.tree.get_mut(pressed_id) {
                                    node.widget_mut().on_event(&WidgetEvent::Click);
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
                            return true;
                        }
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

    let app = SkinApp::new().expect("Failed to load skin");
    let config = RunConfig::default().with_title(&app.title);

    run(app, config);
}
