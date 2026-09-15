use bevy::prelude::*;

use crate::components::{Actor, Collider, Collision, MoveIntent, Player, PlayerID, Solid, Velocity};
use crate::map::load_map;
use crate::rcade::Controller;

pub struct Game;

pub const SCREEN_WIDTH: u32 = 336;
pub const SCREEN_HEIGHT: u32 = 262;

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            // .add_observer(|collision: On<Collision>| {
            //     collision.entity
            // })
            .add_systems(Startup, setup)
            .add_systems(
                FixedUpdate,
                (
                    control_intent,
                    update_players,
                    apply_velocity,
                    set_transforms,
                    set_sprites,
                )
                    .chain(),
            );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0, 0.0),
    ));

    setup_map(&mut commands);
}

fn setup_map(commands: &mut Commands) {
    let map = load_map();
    for bounds in &map.solids {
        commands.spawn((Solid, Collider { bounds: *bounds, remain: Vec2::ZERO, grounded: false }));
    }
    commands.spawn(Player::entity(PlayerID::A, &map));
    commands.spawn(Player::entity(PlayerID::B, &map));
}

fn control_intent(mut query: Query<(&Player, &mut MoveIntent)>, controller: Res<Controller>) {
    for (player, mut intent) in &mut query {
        let mut direction = Vec2::ZERO;
        let control = match player.id {
            PlayerID::A => controller.player_a,
            PlayerID::B => controller.player_b,
        };

        if control.up.pressed { direction.y += 1.0; }
        if control.down.pressed { direction.y -= 1.0; }
        if control.left.pressed { direction.x -= 1.0; }
        if control.right.pressed { direction.x += 1.0; }

        intent.0 = direction;
    }
}

fn update_players(time: Res<Time>, mut query: Query<(&mut Player, &mut Velocity, &MoveIntent, &Collider)>) {
    for (mut player, mut velocity, move_intent, collider) in &mut query {
        player.update(&mut velocity, move_intent, collider, &time);
    }
}

fn apply_velocity(
    time: Res<Time>,
    mut commands: Commands,
    mut actors: Query<(Entity, &Velocity, &mut Collider), (With<Actor>, Without<Solid>)>,
    solids: Query<(Entity, &Collider), With<Solid>>,
) {
    for (entity, velocity, mut collider) in &mut actors {
        collider.remain += **velocity * time.delta_secs();
        let mut movement = collider.remain.round().as_ivec2();
        collider.remain -= movement.as_vec2();
        let sign = movement.signum();
        let others = solids.iter().collect::<Vec<_>>();

        while movement.x != 0 {
            let mut next = *collider;
            next.translate(IVec2::new(sign.x, 0));
            if let Some(target) = next.collides(&others) {
                commands.trigger(Collision { entity, target });
                break;
            }
            collider.bounds = next.bounds;
            movement.x -= sign.x;
        }

        while movement.y != 0 {
            let mut next = *collider;
            next.translate(IVec2::new(0, sign.y));
            if let Some(target) = next.collides(&others) {
                commands.trigger(Collision { entity, target });
                break;
            }
            collider.bounds = next.bounds;
            movement.y -= sign.y;
        }

        collider.update_grounded(&others);
    }
}

fn set_transforms(mut query: Query<(&Collider, &mut Transform)>) {
    for (collider, mut transform) in &mut query {
        let pos = collider.bounds.min;
        transform.translation.x = pos.x as f32;
        transform.translation.y = pos.y as f32;
    }
}

fn set_sprites(mut query: Query<(&Collider, &mut Sprite)>) {
    for (collider, mut sprite) in &mut query {
        sprite.custom_size = Some(collider.bounds.size().as_vec2());
    }
}
