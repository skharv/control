use bevy::prelude::*;

mod game;
mod rollback;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Waiting,
    InGame,
}

pub const FPS: usize = 60;

fn main() {
    let mut app = App::new();

    app.add_plugins((DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
        }),
        rollback::RollbackPlugin,
        game::GamePlugin,
        ))
        .add_state::<AppState>()
        .run();

}
