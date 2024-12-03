use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::PresentMode;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::*;
use scarlet::colormap::{ColorMap, GradientColorMap};
use scarlet::prelude::*;
use stuff::my_color::MyColor;
use stuff::random::random_float;
use stuff::stepping;

struct BallDefaults {
    starting_position: Vec3,
    diameter: f32,
    speed: f32,
    initial_direction: Vec2,
    color: bevy::color::Color,
}

const DEFAULT_WINDOW_WIDTH: f32 = 600.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 600.0;

//const NUM_BALLS: usize = 2000;
const NUM_BALLS: usize = 100;

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
        .add_systems(
            FixedUpdate,
            (apply_velocity, ball_collision_system, ball_warp_system).chain(),
        )
        .run();
}

#[derive(Component)]
struct MyCamera;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Mass(f32);

// #[derive(Event, Default)]
// struct CollisionEvent;
//
// #[derive(Component)]
// struct Collider;

#[derive(Component)]
struct Ball;

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
            Mass(mass),
        ));
    }
}

// Systems
fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn ball_warp_system(mut query: Query<&mut Transform, With<Ball>>, window: Query<&Window>) {
    let window = window.single();
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;

    for mut transform in &mut query {
        let radius = transform.scale.x / 2.0;

        if transform.translation.x > half_width + radius {
            transform.translation.x = -half_width - radius;
        } else if transform.translation.x < -half_width - radius {
            transform.translation.x = half_width + radius;
        }

        if transform.translation.y > half_height + radius {
            transform.translation.y = -half_height - radius;
        } else if transform.translation.y < -half_height - radius {
            transform.translation.y = half_height + radius;
        }
    }
}

fn ball_collision_system(
    mut query: Query<(&mut Transform, &mut Velocity, &Mass), With<Ball>>,
    //    mut collision_events: EventWriter<CollisionEvent>,
) {
    let mut combinations = query.iter_combinations_mut();

    while let Some([(mut t1, mut v1, m1), (mut t2, mut v2, m2)]) = combinations.fetch_next() {
        let x1 = t1.translation.truncate();
        let x2 = t2.translation.truncate();

        // Use the x scaling as the diameter
        let r1 = t1.scale.x / 2.0;
        let r2 = t2.scale.x / 2.0;

        // TODO: check for missing entirely due to speed

        let distance = x1.distance(x2);
        if distance < r1 + r2 {
            // Collision detected
            //collision_events.send(CollisionEvent);

            // Use conservation of momentum to calculate new velocities
            // https://en.wikipedia.org/wiki/Elastic_collision#Two-dimensional_collision_with_two_moving_objects

            // let w1 = (2.0 * m2) / (m1 + m2) * (v1.0 - v2.0).dot(x1 - x2)
            //     / (x1 - x2).length_squared()
            //     * (x1 - x2);
            // let w2 = (2.0 * m1) / (m1 + m2) * (v2.0 - v1.0).dot(x2 - x1)
            //     / (x2 - x1).length_squared()
            //     * (x2 - x1);

            let collision_normal = (x2 - x1).normalize();

            // resolve overlap
            let overlap = (r1 + r2) - distance;
            t1.translation -= (overlap / 2.0) * collision_normal.extend(0.0);
            t2.translation += (overlap / 2.0) * collision_normal.extend(0.0);

            let v_normal = (v1.0 - v2.0).dot(x1 - x2);
            if v_normal > 0.0 {
                // Already moving apart
                continue;
            }

            // let relative_velocity = (v1.0 - v2.0).dot(collision_normal);
            // if relative_velocity > 0.0 {
            //     // Already moving apart
            //     continue;
            // }

            let combined_mass = m1.0 + m2.0;

            // let e = 1.0; // Coefficient of restitution
            // let inverse_mass_sum = (1.0 / m1.0) + (1.0 / m2.0);
            //
            // // Compute impulse
            // let impulse = -(1.0 + e) * relative_velocity / inverse_mass_sum;
            // let impulse_vector = impulse * collision_normal;
            //
            // v1.0 += impulse_vector / m1.0;
            // v2.0 -= impulse_vector / m2.0;

            // let w1 = (2.0 * m2.0) / combined_mass * e * relative_velocity * collision_normal;
            // let w2 = (2.0 * m1.0) / combined_mass * e * relative_velocity * collision_normal;
            //
            // v1.0 -= w1;
            // v2.0 += w2;

            let w1 =
                (2.0 * m2.0) / combined_mass * v_normal / (x1 - x2).length_squared() * (x1 - x2);
            let w2 =
                (2.0 * m1.0) / combined_mass * v_normal / (x2 - x1).length_squared() * (x2 - x1);

            v1.0 -= w1;
            v2.0 -= w2;
        }
    }
}
