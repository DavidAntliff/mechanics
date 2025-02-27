use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::*;
use clap::Parser;
use scarlet::colormap::{ColorMap, GradientColorMap};
use scarlet::prelude::*;
#[allow(unused_imports)]
use stuff::ball::{
    apply_velocity_system, ball_warp_system, naive_ball_collision_system,
    sweep_and_prune_collision_system, sweep_and_prune_collision_system_with_cache,
    update_sorted_balls_cache, Ball, Mass, Velocity,
};
use stuff::my_color::MyColor;
use stuff::random::random_float;

struct BallDefaults {
    starting_position: Vec3,
    diameter: f32,
    mass: f32,
    speed: f32,
    initial_direction: Vec2,
    color: bevy::color::Color,
}

const DEFAULT_WINDOW_WIDTH: f32 = 1920.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 1080.0;

// 1,000 balls, seed = 0, 500 fixed frames @ 1920x1080 = 2,003 collisions
//
// Naive collision detection:
// Performance on MBP M4Pro (on AC power) goes off a cliff around 2750 balls:
//   2500: 115 fps  @ 600x600
//   2725: 63 fps
//   2730: 62 fps
//   2740: 61 fps
//   2750: 57 fps
//   2760: 53 fps
//   2770: 51 fps
//   2780: 39 fps
//   2800: 27 fps
//   4000: 3.3 fps
//  10000: 0.7 fps  @ 1920x1080
//
// Sweep-and-prune collision detection:
//   2500: 190 fps  @ 600x600
//   2800: 170 fps
//   4000: 115 fps
//  10000: 38 fps   @ 1920x1080
//
// Does not seem to be GPU limited, as triangles render in the same time as circles
const DEFAULT_NUM_BALLS: usize = 10000;

const SPEED_SCALING: f32 = 1.0; //20.0;

#[derive(Parser, Resource)]
pub struct Cli {
    #[clap(flatten)]
    pub common: stuff::cli::Cli,

    #[clap(short, long, default_value_t = DEFAULT_NUM_BALLS)]
    num_balls: usize,
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
            //sweep_and_prune_collision_system,
            update_sorted_balls_cache,
            sweep_and_prune_collision_system_with_cache,
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
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
    mut window: Query<&mut Window>,
    _asset_server: Res<AssetServer>,
    cli: Res<Cli>,
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

    let half_width = window.single().width() / 2.0;
    let half_height = window.single().height() / 2.0;

    let color_map = GradientColorMap::new_linear(
        RGBColor {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        },
        RGBColor {
            r: 0.2,
            g: 0.2,
            b: 1.0,
        },
    );

    // Random Balls
    for _ in 0..cli.num_balls {
        spawn_random_ball(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut rng,
            half_width,
            half_height,
            &color_map,
        );
    }
}

fn spawn_random_ball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rng: &mut ResMut<GlobalEntropy<ChaCha8Rng>>,
    half_width: f32,
    half_height: f32,
    color_map: &GradientColorMap<RGBColor>,
) {
    const SPAWN_VELOCITY_MAX: f32 = 100.0 * SPEED_SCALING;

    let max_radius = 5.0;
    let min_radius = 2.5;
    let radius_param = random_float(rng);
    let radius = min_radius + radius_param * (max_radius - min_radius);

    let mass = radius_param * radius_param; // normalised

    let spawn_vec = Vec2::new(2.0 * random_float(rng) - 1.0, 2.0 * random_float(rng) - 1.0);

    let spawn_region_x = spawn_vec.x * half_width;
    let spawn_region_y = spawn_vec.y * half_height;

    // Random velocity
    let spawn_speed = random_float(rng) * SPAWN_VELOCITY_MAX;
    let spawn_direction = random_float(rng) * std::f32::consts::PI * 2.0;
    let spawn_velocity_x = spawn_speed * spawn_direction.cos();
    let spawn_velocity_y = spawn_speed * spawn_direction.sin();

    let color: MyColor = color_map
        .transform_single(radius_transform(spawn_vec.length(), 0.9) as f64)
        .into();

    let ball = BallDefaults {
        starting_position: Vec3::new(spawn_region_x, spawn_region_y, 0.0),
        diameter: radius * 2.0,
        mass,
        speed: spawn_speed,
        initial_direction: Vec2::new(spawn_velocity_x, spawn_velocity_y),
        color: color.into(),
    };

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            //mesh: meshes.add(Triangle2d::default()).into(),
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
