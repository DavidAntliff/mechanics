use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use clap::Parser;
#[allow(unused_imports)]
use stuff::ball::{
    apply_velocity_system, ball_warp_system, sweep_and_prune_collision_system,
    update_sorted_balls_cache, Ball, Mass, Velocity,
};

struct BallDefaults {
    starting_position: Vec3,
    diameter: f32,
    mass: f32,
    speed: f32,
    initial_direction: Vec2,
    color: bevy::color::Color,
}

const DEFAULT_WINDOW_WIDTH: f32 = 600.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 600.0;

const SPEED_SCALING: f32 = 1.0; //20.0;

// A 45 degree deflection requires a trajectory offset of
// 2 * R * sin(22.5 degrees) = 0.7654 * R
const RADIUS: f32 = 10.0;
const OFFSET: f32 = 0.7654 * RADIUS;

const BALL_DEFAULTS: [BallDefaults; 6] = [
    // Horizontal collision
    BallDefaults {
        starting_position: Vec3::new(-100.0, 100.0, 0.0),
        diameter: 2.0 * RADIUS,
        mass: 1.0,
        speed: 100.0 * SPEED_SCALING,
        initial_direction: Vec2::new(1.0, 0.0),
        color: Color::srgb(0.8, 0.7, 0.6),
    },
    BallDefaults {
        starting_position: Vec3::new(100.0, 100.0, 0.0),
        diameter: 2.0 * RADIUS,
        mass: 1.0,
        speed: 100.0 * SPEED_SCALING,
        initial_direction: Vec2::new(-1.0, 0.0),
        color: Color::srgb(0.7, 0.6, 0.8),
    },
    // // Vertical collision
    BallDefaults {
        starting_position: Vec3::new(200.0, 150.0, 0.0),
        diameter: 2.0 * RADIUS,
        mass: 1.0,
        speed: 100.0 * SPEED_SCALING,
        initial_direction: Vec2::new(0.0, -1.0),
        color: Color::srgb(0.8, 0.7, 0.6),
    },
    BallDefaults {
        starting_position: Vec3::new(200.0, -150.0, 0.0),
        diameter: 2.0 * RADIUS,
        mass: 1.0,
        speed: 100.0 * SPEED_SCALING,
        initial_direction: Vec2::new(0.0, 1.0),
        color: Color::srgb(0.7, 0.6, 0.8),
    },
    // Offset collision
    BallDefaults {
        starting_position: Vec3::new(-100.0, -100.0 - OFFSET / 2.0, 0.0),
        diameter: 2.0 * RADIUS,
        mass: 1.0,
        speed: 80.0 * SPEED_SCALING,
        initial_direction: Vec2::new(1.0, 0.0),
        color: Color::srgb(0.8, 0.7, 0.6),
    },
    BallDefaults {
        starting_position: Vec3::new(100.0, -100.0 + OFFSET / 2.0, 0.0),
        diameter: 2.0 * RADIUS,
        mass: 1.0,
        speed: 80.0 * SPEED_SCALING,
        initial_direction: Vec2::new(-1.0, 0.0),
        color: Color::srgb(0.7, 0.6, 0.8),
    },
];

#[derive(Parser, Resource)]
pub struct Cli {
    #[clap(flatten)]
    pub common: stuff::cli::Cli,
}

fn main() {
    let cli = Cli::parse();

    let mut app = stuff::setup::setup(&cli.common);
    app.insert_resource(cli);

    app.add_systems(Startup, setup).add_systems(
        FixedUpdate,
        (
            apply_velocity_system,
            //naive_ball_collision_system,
            (update_sorted_balls_cache, sweep_and_prune_collision_system).chain(),
            ball_warp_system,
        )
            .chain(),
    );

    app.run();
}

#[derive(Component)]
struct MyCamera;

fn radius_transform(u: f32, n: f32) -> f32 {
    u.powf(1.0 / n)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut window: Query<&mut Window>,
    _asset_server: Res<AssetServer>,
) {
    window
        .single_mut()
        .resolution
        .set(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);

    // Camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                //clear_color: ClearColorConfig::None,
                clear_color: ClearColorConfig::Custom(bevy::color::Color::srgb(0.0, 0.0, 0.0)),
                ..default()
            },
            ..default()
        },
        MyCamera,
    ));

    let _half_width = window.single().width() / 2.0;
    let _half_height = window.single().height() / 2.0;

    for ball in BALL_DEFAULTS {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::default()).into(),
                material: materials.add(ball.color),
                transform: Transform::from_translation(ball.starting_position)
                    .with_scale(Vec2::splat(ball.diameter).extend(1.0)),
                ..default()
            },
            Ball,
            Velocity(ball.initial_direction.normalize() * ball.speed),
            Mass(ball.mass),
        ));
    }
}
