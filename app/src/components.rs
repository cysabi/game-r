use bevy::ecs::component::Component;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::map::Map;

#[derive(Component)]
#[require(Sprite::from_color(Color::srgb(0.4, 0.9, 0.7), Vec2::ZERO))]
#[require(Anchor::BOTTOM_LEFT)]
#[require(Actor)]
#[require(MoveIntent)]
pub struct Player {
    pub id: PlayerID,
    pub jump_hold_frames: u8,
    pub hitstop_frames: u8,
}

impl Player {
    const WIDTH: i32 = 8;
    const HEIGHT: i32 = 11;
    const RUN_MAX: f32 = 90.0;
    const RUN_ACCEL: f32 = 1000.0;
    const AIR_ACCEL: f32 = 650.0;
    const GRAVITY: f32 = -900.0;
    const FALL_MAX: f32 = -160.0;
    const FALL_FAST_MAX: f32 = -240.0;
    const JUMP: f32 = 105.0;
    const JUMP_H_BOOST: f32 = 40.0;
    const JUMP_HOLD_DURATION: u8 = 12;

    pub fn entity(id: PlayerID, map: &Map) -> (Self, Collider, Sprite) {
        let spawn = match id {
            PlayerID::A => map.spawn_a,
            PlayerID::B => map.spawn_b,
        };
        let color = match id {
            PlayerID::A => Color::srgb(0.4, 0.9, 0.7),
            PlayerID::B => Color::srgb(0.4, 0.7, 0.9),
        };

        (
            Self { id, jump_hold_frames: 0, hitstop_frames: 0 },
            Collider {
                bounds: IRect {
                    min: spawn,
                    max: spawn + IVec2::new(Player::WIDTH, Player::HEIGHT),
                },
                remain: Vec2::ZERO,
                grounded: false,
            },
            Sprite::from_color(color, Vec2::ZERO),
        )
    }

    pub fn update(&mut self, velocity: &mut Velocity, move_intent: &MoveIntent, collider: &Collider, time: &Res<Time>) {
        velocity.x = Velocity::approach(velocity.x, move_intent.x * Player::RUN_MAX, Player::accel(collider.grounded) * time.delta_secs());
        velocity.y = if collider.grounded { 0.0 } else { (velocity.y + (Player::GRAVITY * time.delta_secs())).max(Player::max_fall(move_intent)) };

        self.update_jump(move_intent, collider.grounded, velocity);
    }

    fn update_jump(&mut self, move_intent: &MoveIntent, grounded: bool, velocity: &mut Velocity) {
        if move_intent.y > 0.0 {
            if grounded {
                velocity.y = Player::JUMP;
                if move_intent.x != 0.0 {
                    velocity.x += move_intent.x.signum() * Player::JUMP_H_BOOST;
                }
                self.jump_hold_frames = Player::JUMP_HOLD_DURATION;
            }
            if self.jump_hold_frames > 0 {
                velocity.y = Player::JUMP;
                self.jump_hold_frames -= 1;
            }
        } else {
            self.jump_hold_frames = 0;
        }
    }

    fn accel(grounded: bool) -> f32 {
        if grounded { Player::RUN_ACCEL } else { Player::AIR_ACCEL }
    }

    fn max_fall(move_intent: &MoveIntent) -> f32 {
        if move_intent.y < 0.0 { Player::FALL_FAST_MAX } else { Player::FALL_MAX }
    }
}

pub enum PlayerID { A, B }

#[derive(Component, Default, Deref, DerefMut)]
#[require(Velocity)]
pub struct MoveIntent(pub Vec2);

#[derive(Component)]
#[require(Sprite::from_color(Color::srgb(0.9, 0.4, 0.6), Vec2::ZERO))]
#[require(Anchor::BOTTOM_LEFT)]
#[require(Collider)]
pub struct Solid;

#[derive(Component, Default)]
#[require(Collider)]
#[require(MoveIntent)]
pub struct Actor;

#[derive(Component, Default, Clone, Copy)]
pub struct Collider {
    pub bounds: IRect,
    pub remain: Vec2,
    pub grounded: bool,
}

impl Collider {
    pub fn translate(&mut self, vec: IVec2) {
        self.bounds.min += vec;
        self.bounds.max += vec;
    }

    pub fn collides(&self, others: &Vec<(Entity, &Collider)>) -> Option<Entity> {
        others.iter().find(|(_, collider)|
            !collider.bounds.intersect(self.bounds).is_empty()).map(|entity| entity.0)
    }

    pub fn update_grounded(&mut self, others: &Vec<(Entity, &Collider)>) {
        let rect = self.bounds.union_point(self.bounds.min - IVec2::Y);
        self.grounded = others.iter().any(|(_, collider)| !collider.bounds.intersect(rect).is_empty());
    }
}

#[derive(EntityEvent)]
pub struct Collision {
    pub entity: Entity,
    pub target: Entity,
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

impl Velocity {
    pub fn approach(from: f32, to: f32, delta: f32) -> f32 {
        from + (to - from).clamp(-delta, delta)
    }
}
