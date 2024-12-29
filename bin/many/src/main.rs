use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::PresentMode;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::*;
use clap::Parser;
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

const NUM_BALLS: usize = 2000;

const SPEED_SCALING: f32 = 1.0; //20.0;

#[derive(Parser)]
pub struct Cli {
    #[arg(short = 't', long = "time", value_name = "SECONDS")]
    duration: Option<f64>,

    #[arg(short = 'f', long = "frames", value_name = "FRAMES")]
    frames: Option<f64>,
}

#[derive(Resource)]
struct BenchmarkTargets {
    duration: Option<f64>,
    frame_count: Option<f64>,
}

fn main() {
    let cli = Cli::parse();
    let seed = [0u8; 32];

    let mut app = App::new();
    app
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
        .add_systems(
            FixedUpdate,
            (
                apply_velocity_system,
                ball_collision_system,
                ball_warp_system,
            )
                .chain(),
        );

    match (cli.duration, cli.frames) {
        (None, None) => (),
        (duration, frame_count) => {
            app.insert_resource(BenchmarkTargets {
                duration,
                frame_count,
            });
            app.add_systems(Update, run_benchmark);
        }
    }

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
    for _ in 0..NUM_BALLS {
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
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    mut rng: &mut ResMut<GlobalEntropy<ChaCha8Rng>>,
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

    let color: MyColor = color_map.transform_single(spawn_vec.length() as f64).into();

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

pub fn run_benchmark(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time<Real>>,
    targets: Res<BenchmarkTargets>,
) {
    if let Some(frame_count) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_COUNT)
        .and_then(|diagnostic| diagnostic.value())
    {
        let duration = time.elapsed_seconds_f64();

        match *targets {
            BenchmarkTargets {
                duration: Some(duration_target),
                frame_count: None,
            } => {
                if duration >= duration_target {
                    println!(
                        "Stopping the app at {:.2} seconds after {:.0} frames, {:.2} fps",
                        duration,
                        frame_count,
                        frame_count / duration
                    );
                    std::process::exit(0);
                }
            }
            BenchmarkTargets {
                duration: None,
                frame_count: Some(frames_target),
            } => {
                if frame_count >= frames_target {
                    println!(
                        "Stopping the app at {:.2} seconds after {:.0} frames, {:.2} fps",
                        duration,
                        frame_count,
                        frame_count / duration
                    );
                    std::process::exit(0);
                }
            }
            _ => {}
        }
    }
}
