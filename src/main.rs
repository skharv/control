use bevy::prelude::*;

mod system;
mod component;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
        }))
        .add_plugins(system::GamePlugin)
        .run();
}
