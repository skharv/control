use bevy::prelude::*;

const INPUT_LEFTCLICK: u8 = 1 << 0;
const INPUT_RIGHTCLICK: u8 = 1 << 1;

pub fn read_local_inputs(
    mut commands: Commands,
    button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    ) {
}
