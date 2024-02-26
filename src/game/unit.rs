use bevy::{prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}};
use bevy_ggrs::prelude::*;
use rand::Rng;
use crate::game::{component, component::*};
use crate::rollback::GameConfig;

const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
const RED: Color = Color::rgb(1.0, 0.0, 0.0);
const PLAYER_COLORS: [Color; 2] = [BLUE, RED];

#[derive(Eq, PartialEq)]
pub enum UnitState{
    Idle,
    Move,
    Hold,
}

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    session: Res<Session<GameConfig>>,
    ) {
    let num_players = match&*session {
        Session::SyncTest(s) => s.num_players(),
        Session::P2P(s) => s.num_players(),
        Session::Spectator(s) => s.num_players(),
    };

    for handle in 0..num_players {
        info!("Spawning units for player {}", handle);
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            let unit_radius = rng.gen_range(5.0..10.0);
            let spawn_point = Vec2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));
            commands.spawn((MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle { radius: unit_radius })),
                material: materials.add(PLAYER_COLORS[handle]),
                transform: Transform::from_xyz(spawn_point.x, spawn_point.y, 0.0),
                ..default()
            },
            Unit,
            Radius { value: unit_radius },
            Velocity { x: 0., y: 0. },
            TargetPosition { x: spawn_point.x, y: spawn_point.y },
            MovementSpeed { value: 10. },
            component::State { state: UnitState::Idle },
            ))
            .add_rollback();
        }
    }
}

pub fn collision(
    mut query: Query<(&mut Transform, &Radius, &mut Velocity, &component::State), With<Unit>>,
    ) {
    let mut combinations = query.iter_combinations_mut();
        while let Some([mut unit1, mut unit2]) = combinations.fetch_next() {
        let distance = unit1.0.translation.distance(unit2.0.translation);
        let combined_radius = unit1.1.value + unit2.1.value;
        if distance < combined_radius {
            let normal = (unit2.0.translation - unit1.0.translation).normalize();
            let separation = combined_radius - distance;
            let mut unit1_difference = unit1.1.value / (unit1.1.value + unit2.1.value);
            let mut unit2_difference = unit2.1.value / (unit1.1.value + unit2.1.value);
            if unit1.3.state == UnitState::Hold {
                unit1_difference = 0.;
                unit2_difference = 1.;
            }
            if unit2.3.state == UnitState::Hold {
                unit2_difference = 0.;
                unit1_difference = 1.;
            }
            unit1.2.x -= normal.x * separation * unit1_difference;
            unit1.2.y -= normal.y * separation * unit1_difference;
            unit2.2.x += normal.x * separation * unit2_difference;
            unit2.2.y += normal.y * separation * unit2_difference;
        }
    }
}

pub fn movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Unit>>,
    ) {
    for (mut transform, mut velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
        velocity.x *= 0.9;
        velocity.y *= 0.9;
    }
}

pub fn select(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Radius), (With<Unit>, Without<Selected>)>,
    already_selected: Query<Entity, With<Selected>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    ) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let (camera, camera_transform) = cameras.single();
        let window = windows.single();
        if let Some(mouse_position) = window.cursor_position() {
            for already_selected_entity in already_selected.iter() {
                commands.entity(already_selected_entity).remove::<Selected>();
            }
            for (entity, transform, radius) in query.iter_mut() {
                if let Some(position) = camera.world_to_viewport(camera_transform, transform.translation) {
                    let distance = position.distance(mouse_position);
                    if distance < radius.value {
                        commands.entity(entity).insert(Selected);
                    }
                }
            }
        }
    }
}

pub fn set_target_position(
    mut query: Query<(&mut TargetPosition, &mut component::State), (With<Unit>, With<Selected>)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    ) {
    if mouse_button_input.just_pressed(MouseButton::Right) {
        let (camera, camera_transform) = cameras.single();
        let window = windows.single();
        if let Some(mouse_position) = window.cursor_position() {
            for (mut target_position, mut state) in query.iter_mut() {
                if let Some(position) = camera.viewport_to_world_2d(camera_transform, mouse_position) {
                    state.state = UnitState::Move;
                    target_position.x = position.x;
                    target_position.y = position.y;
                }
            }
        }
    }
}

pub fn move_towards_target(
    mut query: Query<(&Transform, &TargetPosition, &Radius, &mut Velocity, &MovementSpeed, &mut component::State), With<Unit>>,
    ) {
    for (transform, target_position, radius, mut velocity, movement_speed, mut state) in query.iter_mut() {
        let direction = Vec2::new(target_position.x, target_position.y) - transform.translation.truncate();
        if state.state == UnitState::Move {
            if direction.length() > radius.value {
                let direction = direction.normalize();
                velocity.x += direction.x * movement_speed.value;
                velocity.y += direction.y * movement_speed.value;
            } else {
                state.state = UnitState::Idle;
            }
        } else {
            if direction.length() > radius.value {
                state.state = UnitState::Move;
            }
        }
    }
}

pub fn hold_position(
    mut query: Query<(&mut TargetPosition, &Transform, &mut component::State, &mut Velocity), (With<Unit>, With<Selected>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    ) {
    for (mut target_position, transform, mut state, mut velocity) in query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::KeyD) {
            state.state = UnitState::Hold;
            target_position.x = transform.translation.x;
            target_position.y = transform.translation.y;
            velocity.x = 0.;
            velocity.y = 0.;
        }
    }
}
