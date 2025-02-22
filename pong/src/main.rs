use bevy::{
    app::AppExit, ecs::system::Commands, math::{bounding::{Aabb2d, BoundingVolume, IntersectsVolume}, Vec3Swizzles}, prelude::*, transform, utils::dbg
};
use components::{
    Ball, BallMovement, BallVelocity, 
    Player,
    ReactionBarrier, 
    SpeedUp, 
    Velocity, VelocityAI,
};
use border::BorderPlugin;
use ball::BallPlugin;
use cpu::CPU;
use player::PlayerPlugin;
mod ball;
mod border;
mod components;
mod cpu;
mod player;

const PLAYER_SIZE: (f32, f32) = (20., 125.);
const BALL_SIZE: (f32, f32) = (20., 20.);
const INITAL_SPEED: f32 = 5.;



#[derive(Debug, Clone, Eq, PartialEq, Hash, States)]
enum AppState {
    InGameSinglePlayer,
    Paused,
}
#[derive(Resource, Component, Deref, DerefMut)]
struct Score1 {
    score: usize,
}

#[derive(Resource, Component, Deref, DerefMut)]
struct Score2 {
    score: usize,
}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: bevy::window::WindowResolution::new(1400., 700.),
                title: "pong".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, game_setup)
        .add_plugins((PlayerPlugin, BallPlugin, CPU, BorderPlugin))
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.04)))
        .insert_resource(Score1 { score: 0 })
        .insert_resource(Score2 { score: 0 })
        .insert_state(AppState::InGameSinglePlayer)
        .add_systems(Update, (player_control, cpu_control,  ball_movement, exit_app).run_if(in_state(AppState::InGameSinglePlayer)))
        .add_systems(Update, (play, exit_app).run_if(in_state(AppState::Paused)))
        .run();
}

fn game_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());
    //commands.spawn((
    //    Text::new(""),
    //    TextFont {
    //        font: asset_server.load("fonts/PixeloidSansBold-GOjpP.ttf"),
    //        font_size: 50.0,
    //        ..Default::default()
    //    },
    //    TextColor(Color::WHITE),
    //    Node {
    //        position_type: PositionType::Absolute,
    //        top: Val::Px(5.0),
    //        left: Val::Px(620.0),
    //        ..default()
    //    },
    //    Score1 { score: 0 },
    //));
    //commands.spawn((
    //    Text::new(""),
    //    TextFont {
    //        font: asset_server.load("/assets/fonts/PixeloidSansBold-GOjpP.ttf"),
    //        font_size: 50.0,
    //        ..Default::default()
    //    },
    //    TextColor(Color::WHITE),
    //    Node {
    //        position_type: PositionType::Absolute,
    //        top: Val::Px(5.0),
    //        right: Val::Px(620.0),
    //        ..default()
    //    },
    //    Score2 { score: 0 },
    //));
}

pub fn player_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Player>>,
) {
    if let Ok(_) = query.get_single_mut() {
        if keyboard.pressed(KeyCode::KeyW) {
            for mut transform in &mut query {
                if transform.1.translation.y + 8. < 350. - PLAYER_SIZE.1/2. {
                    transform.0.y = 85.;
                    transform.1.translation.y += 8.;
                }
            }
        }     
        if keyboard.pressed(KeyCode::KeyS) {
            for mut transform in &mut query {
                dbg!(transform.1.scale);
                if transform.1.translation.y + 8. > -350. + PLAYER_SIZE.1/2. {
                    transform.0.y = 85.;
                    transform.1.translation.y -= 8.;
                }
            }
        }     
    }
}

fn cpu_control(
    mut aiquery: Query<(&mut VelocityAI, &mut Transform, &mut ReactionBarrier), Without<Ball>>,
    ballquery: Query<(&BallVelocity, &Transform), With<Ball>>,
) {
    for (ball_velocity, ball_tf) in ballquery.iter() {
        let ball_transform = &ball_tf.translation;
        for (mut ai_velocity, mut ai_transform, mut reaction_bar) in aiquery.iter_mut() {
            let translation = &mut ai_transform.translation;
            let reaction_barrier = reaction_bar.x;
            let mut ease = 0.75;
            //this cpu is kinda fucking OP
            if ball_velocity.x >= 0. {
                if ball_transform.x >= reaction_barrier {
                    if translation.y < ball_transform.y {
                        translation.y += ai_velocity.y / ease;
                        ease += 0.25;
                    } else if translation.y > ball_transform.y {
                        translation.y -= ai_velocity.y / ease;
                        ease += 0.75;
                    } else {
                        translation.y += 0.;
                    }
                }
            } else if ball_velocity.x < 0. {
                if translation.y < -15. {
                    translation.y += ai_velocity.y;
                    ease += 0.25;
                } else if translation.y > 15. {
                    translation.y -= ai_velocity.y;
                    ease += 0.25;
                }
            }
            if ball_transform.x >= (900. - (ai_velocity.y + 3.)) {
                if ai_velocity.y < 18. {
                    ai_velocity.y += 5.;
                }
                if reaction_bar.x > -700. {
                    reaction_bar.x -= 70.;
                }
            }
        }
    }
}

fn ball_movement(
    mut score: ResMut<Score1>,
    mut score2: ResMut<Score2>,
    mut query: Query<
        (
            Entity,
            &mut BallVelocity,
            &mut Transform,
            &BallMovement,
            &mut SpeedUp,
        ),
        With<Ball>,
    >,
) {
    for (_ball_entity, mut ball_velocity, mut ball_transform, ball_movement, mut speedup) in
        query.iter_mut()
    {
        let translation = &mut ball_transform.translation;
        let speedup = &mut speedup.speed;

        translation.y += ball_velocity.y;
        translation.x += ball_velocity.x;

        if ball_movement.auto_despawn {
            if translation.x >= 900. {
                translation.y = 0.;
                translation.x = 0.;
                ball_velocity.y = 0.;
                ball_velocity.x = 5.;
                *speedup = INITAL_SPEED;
                score.score += 1;
            } else if translation.x <= -900. {
                translation.y = 0.;
                translation.x = 0.;
                ball_velocity.y = 0.;
                ball_velocity.x = -5.;
                *speedup = INITAL_SPEED;
                score2.score += 1;
            }
            if translation.y <= -345. {
                ball_velocity.x * 2.;
                ball_velocity.y = (ball_velocity.y * -1.) - 1.;
                translation.y += ball_velocity.y + 5.;
                translation.x += ball_velocity.x;
            } else if translation.y >= 345. {
                ball_velocity.x * 2.;
                ball_velocity.y = (ball_velocity.y * -1.) - 5.;
                translation.y += ball_velocity.y + 5.;
                translation.x += ball_velocity.x;
            }
        }
    }
}

fn update_score1(score: Res<Score1>, mut score_root: Single<Entity, (With<Score1>, With<Text>)>,  mut writer: TextUiWriter) {
    *writer.text(*score_root, 1) = score.to_string();
}
fn update_score2(score: Res<Score2>, mut score_root: Single<Entity, (With<Score2>, With<Text>)>, mut writer: TextUiWriter) {
    *writer.text(*score_root, 1) = score.to_string();
}

fn play(mut keyboard: ResMut<ButtonInput<KeyCode>>, mut next_game_state: ResMut<NextState<AppState>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_game_state.set(AppState::InGameSinglePlayer);
        keyboard.reset(KeyCode::Space);
    }
}

fn pause(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut next_game_state: ResMut<NextState<AppState>>
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_game_state.set(AppState::Paused);
        keyboard.reset(KeyCode::Space);
    }
}

fn exit_app(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
