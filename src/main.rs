//! Loads and renders a glTF file as a scene.

use bevy::{
    input::mouse::MouseMotion,
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};
use std::f32::consts::*;

const CAMERA_ROTATION_SPEED: f32 = 0.3;
const CAMERA_OFFSET: f32 = 20.;

#[derive(Resource)]
pub struct WalkerAnimation(pub Handle<AnimationClip>);

#[derive(Component)]
pub struct CameraController {
    pub rotation: Quat,
}

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (animate_light_direction, animate_walker, handle_camera_input),
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
        },
        Camera3dBundle {
            transform: Transform::from_xyz(20., 12., -25.0)
                .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        },
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/xwing/xwing.gltf#Scene0"),
        ..default()
    });

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/walker/walker.gltf#Scene0"),
        ..default()
    });
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
    time: Res<Time>,
) {
    for (mut cam_transform, mut cam_controller) in camera_controller.iter_mut() {
        for motion in mouse_motion.read() {
            let delta = motion.delta * time.delta_seconds() * CAMERA_ROTATION_SPEED;
            cam_controller.rotation *= Quat::from_euler(EulerRot::XYZ, -delta.x, -delta.y, 0.);
        }
        cam_transform.translation = cam_controller.rotation * Vec3::Z * CAMERA_OFFSET;
        cam_transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
