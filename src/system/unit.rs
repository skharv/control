use bevy::{prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}};
use rand::Rng;
use crate::component::*;

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
    let mut rng = rand::thread_rng();
    for _ in 0..10 {
        let unit_radius = rng.gen_range(5.0..10.0);
        let spawn_point = Vec2::new(rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0));
        commands.spawn((MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle { radius: unit_radius })),
            material: materials.add(Color::ALICE_BLUE),
            transform: Transform::from_xyz(spawn_point.x, spawn_point.y, 0.0),
            ..default()
        },
        Unit,
        Radius { value: unit_radius },
        Velocity { x: 0., y: 0. },
        TargetPosition { x: spawn_point.x, y: spawn_point.y },
        MovementSpeed { value: 10. },
        Idle,
        ));
        info!("Spawned unit with radius: {}", unit_radius);
    }
}

pub fn collision(
    mut query: Query<(&mut Transform, &Radius, &mut Velocity), With<Unit>>,
    ) {
    let mut combinations = query.iter_combinations_mut();
        while let Some([mut unit1, mut unit2]) = combinations.fetch_next() {
        let distance = unit1.0.translation.distance(unit2.0.translation);
        let combined_radius = unit1.1.value + unit2.1.value;
        if distance < combined_radius {
            let center_location = (unit1.0.translation + unit2.0.translation) / 2.;
            warn!("collision detected at {}", center_location);
            let normal = (unit2.0.translation - unit1.0.translation).normalize();
            let separation = combined_radius - distance;
            unit1.2.x -= normal.x * separation;
            unit1.2.y -= normal.y * separation;
            unit2.2.x += normal.x * separation;
            unit2.2.y += normal.y * separation;
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
                        info!("Selected unit with radius: {}", radius.value);
                    }
                }
            }
        }
    }
}

pub fn set_target_position(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TargetPosition), (With<Unit>, With<Selected>)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    ) {
    if mouse_button_input.just_pressed(MouseButton::Right) {
        let (camera, camera_transform) = cameras.single();
        let window = windows.single();
        if let Some(mouse_position) = window.cursor_position() {
            for (entity, mut target_position) in query.iter_mut() {
                if let Some(position) = camera.viewport_to_world_2d(camera_transform, mouse_position) {
                    commands.entity(entity).insert(Move);
                    target_position.x = position.x;
                    target_position.y = position.y;
                }
            }
        }
    }
}

pub fn move_towards_target(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut TargetPosition, &Radius, &mut Velocity, &MovementSpeed), (With<Unit>, With<Move>)>,
    ) {
    for (entity, transform, mut target_position, radius, mut velocity, movement_speed) in query.iter_mut() {
        let direction = Vec2::new(target_position.x, target_position.y) - transform.translation.truncate();
        if direction.length() > radius.value {
            let direction = direction.normalize();
            velocity.x += direction.x * movement_speed.value;
            velocity.y += direction.y * movement_speed.value;
        } else {
            commands.entity(entity).remove::<Move>();
            target_position.x = transform.translation.x;
            target_position.y = transform.translation.y;
        }
    }
}

pub fn set_idle(
    mut commands: Commands,
    query: Query<Entity, (With<Unit>, Without<Move>, Without<Idle>)>,
    ) {
    for entity in query.iter() {
        commands.entity(entity).insert(Idle);
    }
}
