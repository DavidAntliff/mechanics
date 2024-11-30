use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::*;
use rand_core::RngCore;

struct BallDefaults {
    starting_position: Vec3,
    diameter: f32,
    speed: f32,
    initial_direction: Vec2,
    color: Color,
}

const SPEED_SCALING: f32 = 1.0; //20.0;

const BALL_DEFAULTS: [BallDefaults; 4] = [
    BallDefaults {
        starting_position: Vec3::new(-300.0, 300.0, 0.0),
        diameter: 50.0,
        speed: 145.0 * SPEED_SCALING,
        initial_direction: Vec2::new(0.5, -0.5),
        color: Color::srgb(0.8, 0.7, 0.6),
    },
    BallDefaults {
        starting_position: Vec3::new(300.0, 300.0, 0.0),
        diameter: 50.0,
        speed: 155.0 * SPEED_SCALING,
        initial_direction: Vec2::new(-0.5, -0.5),
        color: Color::srgb(0.7, 0.6, 0.8),
    },
    BallDefaults {
        starting_position: Vec3::new(-300.0, -300.0, 0.0),
        diameter: 50.0,
        speed: 100.0 * SPEED_SCALING,
        initial_direction: Vec2::new(0.5, 0.5),
        color: Color::srgb(0.6, 0.8, 0.7),
    },
    BallDefaults {
        starting_position: Vec3::new(300.0, -300.0, 0.0),
        diameter: 50.0,
        speed: 110.0 * SPEED_SCALING,
        initial_direction: Vec2::new(-0.5, 0.5),
        color: Color::srgb(0.7, 0.8, 0.6),
    },
];

fn main() {
    let seed = [0u8; 32];

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EntropyPlugin::<ChaCha8Rng>::with_seed(seed))
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

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Ball;

fn random_float(rng: &mut ResMut<GlobalEntropy<ChaCha8Rng>>) -> f32 {
    // Generate a u32 and normalize it to a floating-point value in [0.0, 1.0]
    let random_u32 = rng.next_u32();
    random_u32 as f32 / u32::MAX as f32
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
    window: Query<&Window>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                //clear_color: ClearColorConfig::None,
                clear_color: ClearColorConfig::Custom(Color::srgb(0.0, 0.0, 0.0)),
                ..default()
            },
            ..default()
        },
        MyCamera,
    ));

    // Balls
    // for ball in BALL_DEFAULTS {
    //     commands.spawn((
    //         MaterialMesh2dBundle {
    //             mesh: meshes.add(Circle::default()).into(),
    //             material: materials.add(ball.color),
    //             transform: Transform::from_translation(ball.starting_position)
    //                 .with_scale(Vec2::splat(ball.diameter).extend(1.0)),
    //             ..default()
    //         },
    //         Ball,
    //         Velocity(ball.initial_direction.normalize() * ball.speed),
    //     ));
    // }

    let half_width = window.single().width() / 2.0;
    let half_height = window.single().height() / 2.0;

    // Random Balls
    let num_balls = 10;
    for i in 0..num_balls {
        let ball = BallDefaults {
            starting_position: Vec3::new(
                (random_float(&mut rng) - 0.5) * window.single().width(),
                (random_float(&mut rng) - 0.5) * window.single().height(),
                0.0,
            ),
            diameter: 10.0 + random_float(&mut rng) * 90.0,
            speed: 100.0 * SPEED_SCALING,
            initial_direction: Vec2::new(
                (random_float(&mut rng) - 0.5),
                (random_float(&mut rng) - 0.5),
            ),
            color: Color::srgb(
                random_float(&mut rng),
                random_float(&mut rng),
                random_float(&mut rng),
            ),
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
    mut query: Query<(&mut Transform, &mut Velocity), With<Ball>>,
    //    mut collision_events: EventWriter<CollisionEvent>,
) {
    let mut combinations = query.iter_combinations_mut();

    while let Some([(mut t1, mut v1), (mut t2, mut v2)]) = combinations.fetch_next() {
        let x1 = t1.translation.truncate();
        let x2 = t2.translation.truncate();

        // Use the x scaling as the diameter
        let r1 = t1.scale.x / 2.0;
        let r2 = t2.scale.x / 2.0;

        // TODO: check for missing entirely due to speed

        let distance = x1.distance(x2);
        if distance < r1 + r2 {
            // Collision detected
            let m1 = r1 * r1 / 1000.0;
            let m2 = r2 * r2 / 1000.0;

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

            let w1 = (2.0 * m2) / (m1 + m2) * v_normal / (x1 - x2).length_squared() * (x1 - x2);
            let w2 = (2.0 * m1) / (m1 + m2) * v_normal / (x2 - x1).length_squared() * (x2 - x1);

            v1.0 -= w1;
            v2.0 -= w2;
        }
    }
}
