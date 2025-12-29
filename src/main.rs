use std::path::Path;

use curvy::{run, App, LoadedSkin, RunConfig, SkinBuilder, UiTree, View, WidgetEvent};
use winit::event::WindowEvent;

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
                        if let Some(hovered) = self.tree.hovered() {
                            self.tree.set_pressed(Some(hovered));
                        }
                    }
                    winit::event::ElementState::Released => {
                        if let Some(pressed_id) = self.tree.pressed() {
                            if let Some(node) = self.tree.get_mut(pressed_id) {
                                node.widget_mut().on_event(&WidgetEvent::Click);
                            }
                        }
                        self.tree.set_pressed(None);
                    }
                }
                true
            }
            _ => false,
        }
    }
}

fn main() {
    let app = SkinApp::new().expect("Failed to load skin");
    let config = RunConfig::default().with_title(&app.title);

    run(app, config);
}
