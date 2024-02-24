use bevy::prelude::*;

mod camera;
mod unit;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (camera::spawn, unit::spawn))
            .add_systems(Update, (unit::collision.after(unit::move_towards_target), unit::select, unit::move_towards_target, unit::set_target_position, unit::movement.after(unit::collision)));
    }
}
