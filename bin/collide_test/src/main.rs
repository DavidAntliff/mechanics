use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::PresentMode;
use stuff::ball::{
    apply_velocity_system, ball_collision_system, ball_warp_system, Ball, Mass, Velocity,
};
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

const SPEED_SCALING: f32 = 1.0; //20.0;

// A 45 degree deflection requires a trajectory offset of
// 2 * R * sin(22.5 degrees) = 0.7654 * R
const RADIUS: f32 = 10.0;
const OFFSET: f32 = 0.7654 * RADIUS;

const BALL_DEFAULTS: [BallDefaults; 4] = [
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
    // Offset collision
    BallDefaults {
        starting_position: Vec3::new(-100.0, -100.0 - OFFSET / 2.0, 0.0),
        diameter: 2.0 * RADIUS,
        mass: 1.0,
        speed: 100.0 * SPEED_SCALING,
        initial_direction: Vec2::new(1.0, 0.0),
        color: Color::srgb(0.8, 0.7, 0.6),
    },
    BallDefaults {
        starting_position: Vec3::new(100.0, -100.0 + OFFSET / 2.0, 0.0),
        diameter: 2.0 * RADIUS,
        mass: 1.0,
        speed: 100.0 * SPEED_SCALING,
        initial_direction: Vec2::new(-1.0, 0.0),
        color: Color::srgb(0.7, 0.6, 0.8),
    },
];

fn main() {
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
        // // Add diagnostics
        // .add_plugins((
        //     FrameTimeDiagnosticsPlugin::default(),
        //     LogDiagnosticsPlugin::default(),
        // ))
        .add_systems(Startup, setup)
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
