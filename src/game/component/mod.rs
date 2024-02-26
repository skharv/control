use bevy::prelude::*;
use crate::game::unit::UnitState;

#[derive(Component)]
pub struct Player {
    handle: usize,
}

#[derive(Component)]
pub struct Radius{
    pub value: f32,
}

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct Velocity{
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct TargetPosition{
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct MovementSpeed{
    pub value: f32,
}

#[derive(Component)]
pub struct State {
    pub state: UnitState,
}
