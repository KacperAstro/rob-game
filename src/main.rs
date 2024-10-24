use bevy::{
    color::palettes::css::WHITE, math::VectorSpace, prelude::*, sprite::MaterialMesh2dBundle,
};
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (apply_kinematics/* , update_camera*/))
        .add_systems(Update, (animate_sprites, get_player_input))
        .run();
}

// Tags
#[derive(Component)]
struct PlayerTag;

#[derive(Component)]
struct ColliderTag;

#[derive(Component)]
struct MainCameraTag;

// Other structs/enums
#[derive(Debug)]
enum FacingDirection {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

struct AnimIndices {
    left: usize,
    right: usize,
    up: usize,
    down: usize,
}

// Components
#[derive(Component)]
struct FaceDirection(FacingDirection);

#[derive(Component)]
struct MoveSettings {
    is_walking: bool,
    speed: f32,
    accel: f32,
    fric: f32,
}

#[derive(Component)]
struct CameraValues {
    lerp_factor: f32,
}

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct AnimationInd {
    walk: AnimIndices,
    idle: AnimIndices,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load Textures
    let sprite_texture: Handle<Image> = asset_server.load("spritesheet.png");

    let atlas = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 8, None, None);
    let texture_atlas_layouts = texture_atlas_layouts.add(atlas);

    let animation_indices = AnimationInd {
        walk: AnimIndices {
            right: 0,
            left: 8,
            up: 24,
            down: 16,
        },
        idle: AnimIndices {
            right: 32,
            left: 40,
            up: 56,
            down: 48,
        },
    };
    // Camera Spawn
    commands.spawn((
        Camera2dBundle::default(),
        MainCameraTag,
        CameraValues { lerp_factor: 2.0 },
    ));

    // UI
    commands.spawn(
        TextBundle::from_section("Welcome", TextStyle::default()).with_style(Style {
            position_type: PositionType::Relative,
            ..default()
        }),
    );

    // Box
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::new(45.0, 45.0)).into(),
            material: materials.add(Color::from(WHITE)),
            transform: Transform {
                translation: Vec3::new(200., 200., 0.),
                ..default()
            },
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(22.5, 22.5),
        ColliderTag,
    ));

    // Box
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::new(45.0, 45.0)).into(),
            material: materials.add(Color::from(WHITE)),
            transform: Transform {
                translation: Vec3::new(-200., 200., 0.),
                ..default()
            },
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(22.05, 22.5),
        ColliderTag,
    ));

    // Player
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(3.)),
            texture: sprite_texture,
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layouts,
            index: 0,
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        PlayerTag,
        MoveSettings {
            is_walking: false,
            speed: 5.0,
            accel: 20.0,
            fric: 15.0,
        },
        FaceDirection(FacingDirection::DOWN),
        Velocity(Vec2::ZERO),
        RigidBody::KinematicPositionBased,
        Collider::ball(7.0),
        KinematicCharacterController::default(),
    ));
}

fn get_player_input(
    mut player_vel: Query<(&mut Velocity, &mut MoveSettings, &mut FaceDirection), With<PlayerTag>>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let (mut player_vel, mut move_settings, mut face_direction) = player_vel.single_mut();
    let mut input_vector = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyA) {
        input_vector.x = -1.0;
        face_direction.0 = FacingDirection::LEFT;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        input_vector.x = 1.0;
        face_direction.0 = FacingDirection::RIGHT;
    }
    if keyboard.pressed(KeyCode::KeyW) {
        input_vector.y = 1.0;
        face_direction.0 = FacingDirection::UP;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        input_vector.y = -1.0;
        face_direction.0 = FacingDirection::DOWN;
    }

    input_vector = input_vector.normalize_or_zero();

    if input_vector != Vec2::ZERO {
        move_settings.is_walking = true;
        player_vel.0 = player_vel.0.lerp(
            input_vector * move_settings.speed,
            move_settings.accel * time.delta_seconds(),
        );
    } else {
        move_settings.is_walking = false;

        player_vel.0 = player_vel
            .0
            .lerp(Vec2::ZERO, move_settings.fric * time.delta_seconds());
    }
}

fn apply_kinematics(mut entity_transforms: Query<(&mut KinematicCharacterController, &Velocity)>) {
    for (mut transform, vel) in &mut entity_transforms {
        transform.translation = Some(vel.0);
    }
}

fn animate_sprites(
    time: Res<Time>,
    mut sprites: Query<(
        &AnimationInd,
        &mut AnimationTimer,
        &mut TextureAtlas,
        &MoveSettings,
        &FaceDirection,
    )>,
) {
    for (indices, mut timer, mut atlas, move_settings, face_direction) in &mut sprites {
        timer.tick(time.delta());
        if timer.just_finished() {
            if move_settings.is_walking {
                let dir_offset = match face_direction.0 {
                    FacingDirection::LEFT => indices.walk.left,
                    FacingDirection::RIGHT => indices.walk.right,
                    FacingDirection::UP => indices.walk.up,
                    FacingDirection::DOWN => indices.walk.down,
                };

                atlas.index = (atlas.index + 1) % 8 + dir_offset;
            } else {
                let dir_offset = match face_direction.0 {
                    FacingDirection::LEFT => indices.idle.left,
                    FacingDirection::RIGHT => indices.idle.right,
                    FacingDirection::UP => indices.idle.up,
                    FacingDirection::DOWN => indices.idle.down,
                };

                atlas.index = (atlas.index + 1) % 4 + dir_offset;
            }
        }
    }
}

fn update_camera(
    mut camera: Query<(&mut Transform, &CameraValues), (With<MainCameraTag>, Without<PlayerTag>)>,
    player: Query<&Transform, (With<PlayerTag>, Without<MainCameraTag>)>,
    time: Res<Time>,
) {
    let (mut camera_transform, camera_val) = camera.single_mut();
    let player_transform = player.single();

    let Vec3 { x, y, .. } = player_transform.translation;
    let dir = Vec3::new(x, y, camera_transform.translation.z);

    camera_transform.translation = camera_transform
        .translation
        .lerp(dir, time.delta_seconds() * camera_val.lerp_factor);
}
