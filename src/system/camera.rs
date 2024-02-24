use bevy::prelude::*;

pub fn spawn(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1,
            ..default()
        },
        ..default()
    });
}
