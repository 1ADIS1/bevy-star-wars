use bevy::prelude::*;

pub const PLAYER_MOVEMENT_SPEED: f32 = 200.;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
            // Comment this if you're not using pixel art.
            // This sets image filtering to nearest
            // This is done to prevent textures with low resolution (e.g. pixel art) from being blurred
            // by linear filtering.
            ImagePlugin::default_nearest(),
        ))
        .add_systems(Startup, (spawn_2d_camera, spawn_player))
        .add_systems(Update, move_player)
        .run();
}

fn spawn_2d_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle { ..default() });
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("art/ball.png"),
            ..default()
        },
        Player,
    ));
}

/// Handles the player movement each frame by updating it's **transform** component.
fn move_player(
    mut player_query: Query<&mut Transform, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::A) {
            direction.x -= 1.;
        }
        if keyboard_input.pressed(KeyCode::D) {
            direction.x += 1.;
        }
        if keyboard_input.pressed(KeyCode::W) {
            direction.y += 1.;
        }
        if keyboard_input.pressed(KeyCode::S) {
            direction.y -= 1.;
        }

        let direction = direction.normalize_or_zero();
        player_transform.translation += direction * PLAYER_MOVEMENT_SPEED * time.delta_seconds();
    }
}
