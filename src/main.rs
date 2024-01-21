mod landscape;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    pbr::DirectionalLightShadowMap,
    prelude::*,
};
use bevy_inspector_egui::quick::AssetInspectorPlugin;
use landscape::{LandscapeMaterial, LandscapePlugin, MoveWithLandscape};
use std::f32::consts::*;

const CAMERA_ROTATION_SPEED: f32 = 0.3;

/// measured in seconds
const WALKER_SPAWN_INTERVAL: f32 = 2.0;

#[derive(Resource)]
pub struct WalkerAnimation(pub Handle<AnimationClip>);

#[derive(Component)]
pub struct CameraController {
    pub rotation: Quat,
    pub zoom: f32,
}

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins((
            DefaultPlugins,
            LandscapePlugin,
            AssetInspectorPlugin::<LandscapeMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                animate_light_direction,
                animate_walker,
                handle_camera_input,
                spawn_walkers,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(WalkerAnimation(
        asset_server.load("models/walker/walker.gltf#Animation0"),
    ));

    commands.spawn((
        CameraController {
            rotation: Quat::IDENTITY,
            zoom: 45.,
        },
        Camera3dBundle {
            transform: Transform::from_xyz(0.7, 20., 40.0)
                .looking_at(Vec3::new(0., 0.3, 0.), Vec3::Y),
            ..default()
        },
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false, // TODO: set to true on release
            ..default()
        },
        ..default()
    });

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/xwing/xwing.gltf#Scene0"),
        ..default()
    });
}

fn spawn_walkers(
    mut commands: Commands,
    mut timer: Local<f32>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    *timer -= time.delta_seconds();
    if *timer >= 0. {
        return;
    }
    *timer += WALKER_SPAWN_INTERVAL;

    commands.spawn((
        MoveWithLandscape {},
        SceneBundle {
            scene: asset_server.load("models/walker/walker.gltf#Scene0"),
            transform: Transform::from_xyz(-30., -20., 0.),
            ..default()
        },
    ));
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

pub fn animate_walker(
    walker_animation: Res<WalkerAnimation>,
    mut walker_animation_players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut animation_player in walker_animation_players.iter_mut() {
        animation_player
            .play(walker_animation.0.clone_weak())
            .repeat();
    }
}

pub fn handle_camera_input(
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera_controller: Query<(&mut Transform, &mut CameraController)>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    time: Res<Time>,
) {
    for (mut cam_transform, mut cam_controller) in camera_controller.iter_mut() {
        for wheel in mouse_wheel.read() {
            cam_controller.zoom += wheel.y;
        }

        if mouse_buttons.pressed(MouseButton::Right) {
            for motion in mouse_motion.read() {
                let delta = motion.delta * time.delta_seconds() * CAMERA_ROTATION_SPEED;
                cam_controller.rotation *= Quat::from_euler(EulerRot::XYZ, -delta.x, -delta.y, 0.);
            }
        }

        cam_transform.translation = cam_controller.rotation * Vec3::Z * cam_controller.zoom;
        cam_transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
