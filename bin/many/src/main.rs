use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::PresentMode;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::*;
use scarlet::colormap::{ColorMap, GradientColorMap};
use scarlet::prelude::*;
use stuff::ball::{
    apply_velocity_system, ball_collision_system, ball_warp_system, Ball, Mass, Velocity,
};
use stuff::my_color::MyColor;
use stuff::random::random_float;
use stuff::stepping;

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

//const NUM_BALLS: usize = 2000;
const NUM_BALLS: usize = 500;

const SPEED_SCALING: f32 = 1.0; //20.0;

fn main() {
    let seed = [0u8; 32];

    App::new()
        // Disable VSYNC
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // Turn off vsync to maximize CPU/GPU usage
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        // Enable stepping when compiled with '--features=bevy_debug_stepping'
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .at(Val::Percent(35.0), Val::Percent(50.0)),
        )
        // See the random number generator
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::with_seed(seed))
        // Add diagnostics
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input,))
        .add_systems(
            FixedUpdate,
            (
                apply_velocity_system,
                ball_collision_system,
                ball_warp_system,
            )
                .chain(),
        )
        .run();
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

    //let spawn_radius_max = 2.0 * _half_width / 3.0;
    let spawn_radius_max = _half_width / 2.0;
    let spawn_velocity_max = 100.0 * SPEED_SCALING;

    // Random Balls
    for _ in 0..NUM_BALLS {
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

        // let color = Color::srgb(
        //     random_float(&mut rng),
        //     random_float(&mut rng),
        //     random_float(&mut rng),
        // );
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
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        let width = window.single().width();
        let height = window.single().height();

        let radius = 0.8 * f32::min(width, height) / 2.0;
        let angle = random_float(&mut rng) * 2.0 * std::f32::consts::PI;
        let spawn_x = radius * angle.cos();
        let spawn_y = radius * angle.sin();
        // dbg!(radius);
        // dbg!(angle);
        // dbg!((spawn_x, spawn_y));

        let ball = BallDefaults {
            starting_position: Vec3::new(spawn_x, spawn_y, 0.0),
            diameter: 40.0,
            mass: 20.0,
            speed: 150.0 * SPEED_SCALING,
            initial_direction: Vec2::new(-spawn_x, -spawn_y),
            color: bevy::prelude::Color::srgb(1.0, 0.0, 0.0),
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
