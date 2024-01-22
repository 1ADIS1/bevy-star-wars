mod landscape;
mod laser;

use bevy::{
    core_pipeline::bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
    input::mouse::{MouseMotion, MouseWheel},
    math::vec3,
    pbr::DirectionalLightShadowMap,
    prelude::*,
};
use bevy_inspector_egui::quick::AssetInspectorPlugin;
use landscape::*;
use laser::*;
use rand::Rng;
use std::f32::consts::*;

const CAMERA_ROTATION_SPEED: f32 = 0.3;

/// measured in seconds
const MOVABLE_SPAWN_INTERVAL: f32 = 1.0;
const X_FAR_SPAWN: f32 = 400.;

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
            LaserPlugin,
            AssetInspectorPlugin::<LandscapeMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                animate_light_direction,
                animate_walker,
                handle_camera_input,
                spawn_movables,
                move_plane,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PlaneSettings {
        move_interval: 1.3,
        box_area: 6.0,
        speed: 1.5,
        wobble_speed: 5.0,
        rotation_speed: 0.7,
    });

    // Make background color the same as fog
    commands.insert_resource(ClearColor(Color::rgb(0.7, 0.92, 0.96)));

    commands.insert_resource(WalkerAnimation(
        asset_server.load("models/walker/walker.gltf#Animation0"),
    ));

    commands.spawn((
        CameraController {
            rotation: Quat::IDENTITY,
            zoom: 45.,
        },
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(0.7, 20., 40.0)
                .looking_at(Vec3::new(0., 0.3, 0.), Vec3::Y),
            ..default()
        },
        FogSettings {
            color: Color::rgb_u8(117, 202, 215),
            directional_light_color: Color::WHITE,
            directional_light_exponent: 30.0,
            falloff: FogFalloff::Linear {
                start: 0.0,
                end: LANDSCAPE_SIZE_HALF,
            },
        },
        BloomSettings {
            intensity: 1.0,
            low_frequency_boost: 0.5,
            low_frequency_boost_curvature: 0.5,
            high_pass_frequency: 0.5,
            prefilter_settings: BloomPrefilterSettings {
                threshold: 3.0,
                threshold_softness: 0.6,
            },
            composite_mode: BloomCompositeMode::Additive,
        },
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false, // TODO
            ..default()
        },
        ..default()
    });

    commands.spawn((
        PlaneMovement {
            target_pos: Vec3::ZERO,
            timer: 0.0,
        },
        GunAwaitingToBeSpawned { color: Color::RED },
        SceneBundle {
            scene: asset_server.load("models/xwing/xwing.gltf#Scene0"),
            ..default()
        },
    ));
}

fn randomise_position(x_close_spawn: f32) -> Transform {
    let mut rng = rand::thread_rng();
    let flip = (rng.gen_range(0..=1) * 2 - 1) as f32;

    Transform::from_xyz(
        rng.gen_range(x_close_spawn..X_FAR_SPAWN) * flip,
        -20.,
        -LANDSCAPE_SIZE_HALF,
    )
    .with_rotation(Quat::from_rotation_y(rng.gen_range(0.0..PI * 2.)))
}

fn spawn_movables(
    mut commands: Commands,
    mut timer: Local<f32>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    *timer -= time.delta_seconds();
    if *timer >= 0. {
        return;
    }
    *timer += MOVABLE_SPAWN_INTERVAL;

    let mut rng = rand::thread_rng();
    match rng.gen_range(0..=100) {
        0..=70 => {
            commands.spawn((
                GunAwaitingToBeSpawned {
                    color: Color::GREEN,
                },
                MoveWithLandscape {},
                SceneBundle {
                    scene: asset_server.load("models/walker/walker.gltf#Scene0"),
                    transform: randomise_position(36.),
                    ..default()
                },
            ));
        }
        71..=91 => {
            commands.spawn((
                MoveWithLandscape {},
                SceneBundle {
                    scene: asset_server.load("models/desert_rock_column/scene.gltf#Scene0"),
                    transform: randomise_position(30.),
                    ..default()
                },
            ));
        }
        _ => {
            commands.spawn((
                MoveWithLandscape {},
                SceneBundle {
                    scene: asset_server.load("models/desert_cliff_6/scene.gltf#Scene0"),
                    transform: randomise_position(90.),
                    ..default()
                },
            ));
        }
    }
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

#[derive(Component)]
pub struct PlaneMovement {
    target_pos: Vec3,
    timer: f32,
}

#[derive(Resource, Reflect)]
pub struct PlaneSettings {
    wobble_speed: f32,
    rotation_speed: f32,
    move_interval: f32,
    box_area: f32,
    speed: f32,
}

fn move_plane(
    mut query: Query<(&mut Transform, &mut PlaneMovement)>,
    time: Res<Time>,
    plane_settings: Res<PlaneSettings>,
) {
    let mut rng = rand::thread_rng();
    for (mut transform, mut plane_movement) in query.iter_mut() {
        plane_movement.timer -= time.delta_seconds();
        let dir =
            (transform.translation - (plane_movement.target_pos + Vec3::Z * 10.0)).normalize();
        let rot = Quat::from_rotation_arc(transform.forward(), dir);
        transform.rotation = transform
            .rotation
            .lerp(rot, plane_settings.rotation_speed * time.delta_seconds());

        let scaled_time = time.elapsed_seconds() * plane_settings.wobble_speed;
        let wobble = vec3(scaled_time.sin(), scaled_time.cos(), 0.0);
        let target_pos = plane_movement.target_pos + wobble;
        if plane_movement.timer < 0.0 {
            plane_movement.timer += plane_settings.move_interval;
            plane_movement.target_pos = vec3(
                rng.gen_range(-plane_settings.box_area..plane_settings.box_area),
                rng.gen_range(-plane_settings.box_area..plane_settings.box_area),
                rng.gen_range(-plane_settings.box_area..plane_settings.box_area),
            );
        }
        transform.translation = transform
            .translation
            .lerp(target_pos, plane_settings.speed * time.delta_seconds());
    }
}
