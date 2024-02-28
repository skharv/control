use bevy::prelude::*;

mod component;
mod camera;
mod input;
pub mod unit;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera::spawn)
            .add_systems(OnEnter(crate::AppState::InGame), unit::spawn)
            .add_systems(Update, (
                    unit::collision.after(unit::move_towards_target),
                    unit::move_towards_target,
                    unit::movement.after(unit::collision),
                    unit::hold_position,
                    unit::select,
                    unit::set_target_position));
    }
}
