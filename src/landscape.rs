use bevy::{
    prelude::*,
    render::{
        render_resource::{AddressMode, AsBindGroup, ShaderRef},
        texture::{ImageSampler, ImageSamplerDescriptor},
    },
};

pub const LANDSCAPE_SIZE: f32 = 1200.;
pub const LANDSCAPE_SIZE_HALF: f32 = LANDSCAPE_SIZE / 2.;

#[derive(Component)]
pub struct MoveWithLandscape;

#[derive(Resource)]
pub struct CurrentLandscapeMaterial(pub Handle<LandscapeMaterial>);

pub struct LandscapePlugin;

impl Plugin for LandscapePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MaterialPlugin::<LandscapeMaterial>::default(),))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_time_uniform,
                    set_textures_repeating,
                    move_with_landscape,
                ),
            );
    }
}

#[derive(Reflect, Asset, AsBindGroup, Debug, Clone)]
pub struct LandscapeMaterial {
    #[uniform(0)]
    time: f32,
    #[uniform(0)]
    speed: f32,
    #[uniform(0)]
    terrain_height: f32,
    #[uniform(0)]
    terrain_size: f32,
    #[uniform(0)]
    uv_scaling: f32,
    #[uniform(0)]
    quad_size: f32,

    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for LandscapeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/landscape.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/landscape.wgsl".into()
    }
}

pub fn update_time_uniform(mut materials: ResMut<Assets<LandscapeMaterial>>, time: Res<Time>) {
    for material in materials.iter_mut() {
        material.1.time = time.elapsed_seconds();
    }
}

/// Used to make landscape repeating infinitely.
fn set_textures_repeating(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    for event in texture_events.read() {
        match event {
            AssetEvent::Added { id } => {
                let texture = textures.get_mut(*id).expect("Couldn't find the texture!");
                texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: AddressMode::Repeat.into(),
                    address_mode_v: AddressMode::Repeat.into(),
                    ..default()
                });
            }
            _ => (),
        }
    }
}

fn move_with_landscape(
    mut commands: Commands,
    mut movable_query: Query<(&mut Transform, Entity), With<MoveWithLandscape>>,
    materials: Res<Assets<LandscapeMaterial>>,
    current_landscape: Res<CurrentLandscapeMaterial>,
    time: Res<Time>,
) {
    let landscape_material = materials
        .get(&current_landscape.0)
        .expect("Couldn't find landscape material.");
    let delta = landscape_material.speed * time.delta_seconds();

    for (mut transform, entity) in movable_query.iter_mut() {
        transform.translation.z -= delta;

        if transform.translation.z >= LANDSCAPE_SIZE_HALF {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LandscapeMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let quad = shape::Plane {
        size: LANDSCAPE_SIZE,
        subdivisions: 1000,
    };

    let material = materials.add(LandscapeMaterial {
        time: 0.,
        speed: -103.,
        terrain_height: 14.,
        terrain_size: 2.,
        uv_scaling: 1.,
        quad_size: LANDSCAPE_SIZE,
        color_texture: asset_server.load("textures/ground.png"),
    });

    commands.insert_resource(CurrentLandscapeMaterial(material.clone()));

    commands.spawn(MaterialMeshBundle {
        material,
        mesh: meshes.add(quad.into()),
        transform: Transform::from_xyz(0., -25., 0.),
        ..default()
    });
}
