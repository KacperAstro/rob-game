use bevy::{
    color::palettes::css::WHITE, math::VectorSpace, prelude::*, sprite::MaterialMesh2dBundle,
};
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (get_player_input,))
        .add_systems(Update, animate_sprites)
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
}

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
        ColliderTag,
        Collider::cuboid(22.5, 22.5),
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
        ColliderTag,
        Collider::cuboid(22.5, 22.5),
    ));

    let camera = commands
        .spawn((
            Camera2dBundle {
                transform: Transform::from_scale(Vec3::splat(1. / 3.)),
                ..default()
            },
            MainCameraTag,
        ))
        .id();

    // Player
    let player = commands
        .spawn((
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
                speed: 150.0,
            },
            FaceDirection(FacingDirection::DOWN),
            Velocity::zero(),
            GravityScale(0.),
            LockedAxes::ROTATION_LOCKED,
            RigidBody::Dynamic,
            Damping {
                linear_damping: 10.,
                ..default()
            },
            ExternalImpulse::default(),
            Collider::ball(8.),
            ColliderMassProperties::Mass(5.),
        ))
        .id();

    commands.entity(player).add_child(camera);
}

fn get_player_input(
    mut player_vel: Query<
        (&mut ExternalImpulse, &mut MoveSettings, &mut FaceDirection),
        With<PlayerTag>,
    >,
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
    } else {
        move_settings.is_walking = false;
    }

    player_vel.impulse = input_vector * move_settings.speed;
    player_vel.torque_impulse = 200.0;
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
