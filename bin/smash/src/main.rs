use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::*;
use clap::Parser;
use scarlet::colormap::{ColorMap, GradientColorMap};
use scarlet::prelude::*;
use stuff::ball::SortedBallsCache;
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

const DEFAULT_WINDOW_WIDTH: f32 = 800.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 600.0;

//const NUM_BALLS: usize = 2000;
const DEFAULT_NUM_BALLS: usize = 2000;

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

    app.add_systems(Startup, setup)
        .add_systems(Update, (handle_input,))
        .add_systems(
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

    let _half_width = window.single().width() / 2.0;
    let _half_height = window.single().height() / 2.0;

    let spawn_radius_max = 2.0 * _half_width / 3.0;
    let spawn_velocity_max = 100.0 * SPEED_SCALING;

    // Random Balls
    for _ in 0..cli.num_balls {
        let max_radius = 5.0;
        let min_radius = 2.5;
        let radius_param = random_float(&mut rng);
        let radius = min_radius + radius_param * (max_radius - min_radius);

        let mass = radius_param * radius_param; // normalised

        // Spawn in a circular region, spread out a bit
        let spawn_radius_norm = radius_transform(random_float(&mut rng), 2.0);
        let spawn_region_radius = spawn_radius_max * spawn_radius_norm;

        let spawn_region_angle = random_float(&mut rng) * std::f32::consts::PI * 2.0;
        let spawn_region_x = spawn_region_radius * spawn_region_angle.cos();
        let spawn_region_y = spawn_region_radius * spawn_region_angle.sin();

        // Random velocity
        let spawn_speed = random_float(&mut rng) * spawn_velocity_max;
        let spawn_direction = random_float(&mut rng) * std::f32::consts::PI * 2.0;
        let spawn_velocity_x = spawn_speed * spawn_direction.cos();
        let spawn_velocity_y = spawn_speed * spawn_direction.sin();

        // TODO remove
        let spawn_speed = 0.0;

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
        let color: MyColor = color_map.transform_single(spawn_radius_norm as f64).into();

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

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
    window: Query<&Window>,
    mut ball_cache: ResMut<SortedBallsCache>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        let width = window.single().width();
        let height = window.single().height();

        let radius = 1.2 * f32::min(width, height) / 2.0;
        let angle = random_float(&mut rng) * 2.0 * std::f32::consts::PI;
        let spawn_x = radius * angle.cos();
        let spawn_y = radius * angle.sin();
        // dbg!(radius);
        // dbg!(angle);
        // dbg!((spawn_x, spawn_y));

        let ball = BallDefaults {
            starting_position: Vec3::new(spawn_x, spawn_y, 0.0),
            diameter: 40.0,
            mass: 10.0,
            speed: 250.0 * SPEED_SCALING,
            initial_direction: Vec2::new(-spawn_x, -spawn_y),
            color: bevy::prelude::Color::srgb(1.0, 0.0, 0.0),
        };

        let transform = Transform::from_translation(ball.starting_position)
            .with_scale(Vec2::splat(ball.diameter).extend(1.0));

        let entity_commands = commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::default()).into(),
                material: materials.add(ball.color),
                transform,
                ..default()
            },
            Ball,
            Velocity(ball.initial_direction.normalize() * ball.speed),
            Mass(ball.mass),
        ));

        ball_cache.add(entity_commands.id(), transform);
    }
}
