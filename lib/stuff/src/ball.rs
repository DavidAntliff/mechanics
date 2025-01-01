use bevy::math::Vec2;
use bevy::prelude::{Component, Deref, DerefMut, Query, Res, Time, Transform, Window, With};

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Mass(pub f32);

#[derive(Component)]
pub struct Ball;

// Systems
pub fn apply_velocity_system(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    // In FixedUpdate context, time.delta_seconds() is the fixed time step.
    // https://bevy-cheatbook.github.io/fundamentals/fixed-timestep.html

    // First-order explicit Euler update
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0.extend(0.0) * time.delta_seconds();
        // transform.translation.x += velocity.x * time.delta_seconds();
        // transform.translation.y += velocity.y * time.delta_seconds();
    }
}

pub fn ball_warp_system(mut query: Query<&mut Transform, With<Ball>>, window: Query<&Window>) {
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

pub fn naive_ball_collision_system(
    mut query: Query<(&mut Transform, &mut Velocity, &Mass), With<Ball>>,
) {
    // Naive O(n^2) collision detection, comparing every particle with every other particle
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
            perform_collision(&mut t1, &mut v1, m1, &mut t2, &mut v2, m2);
        }
    }
}

pub fn sweep_and_prune_collision_system(
    mut query: Query<(&mut Transform, &mut Velocity, &Mass), With<Ball>>,
) {
    // Sweep and prune collision detection
    // https://leanrada.com/notes/sweep-and-prune/

    // Sort particles by left x bound
    // TODO exploit temporal coherence by persisting sorted order
    let mut particles = query.iter_mut().collect::<Vec<_>>();

    // O(n log n)
    particles.sort_by(|a, b| {
        let ax = a.0.translation.x - a.0.scale.x / 2.0; // radius is scale / 2.0
        let bx = b.0.translation.x - b.0.scale.x / 2.0;
        ax.partial_cmp(&bx).unwrap()
    });

    // O(n + m)
    for i in 0..particles.len() {
        let (left, right) = particles.split_at_mut(i + 1);

        let (t1, v1, m1) = &mut left[i];
        let right1 = t1.translation.x + t1.scale.x / 2.0;

        // O(1) at best; O(m/n) on average; O(n) at worst
        for (t2, v2, m2) in right {
            let left2 = t2.translation.x - t2.scale.x / 2.0;

            if left2 > right1 {
                break;
            }

            let x1 = t1.translation.truncate();
            let x2 = t2.translation.truncate();

            // Use the x scaling as the diameter
            let r1 = t1.scale.x / 2.0;
            let r2 = t2.scale.x / 2.0;

            let distance = x1.distance(x2);
            if distance < r1 + r2 {
                perform_collision(t1, v1, m1, t2, v2, m2);
            }
        }
    }

    // Final: O(n log n + m).
}

fn perform_collision(
    t1: &mut Transform,
    v1: &mut Velocity,
    m1: &Mass,
    t2: &mut Transform,
    v2: &mut Velocity,
    m2: &Mass,
) {
    // Use conservation of momentum to calculate new velocities
    // https://en.wikipedia.org/wiki/Elastic_collision#Two-dimensional_collision_with_two_moving_objects

    // let w1 = (2.0 * m2) / (m1 + m2) * (v1.0 - v2.0).dot(x1 - x2)
    //     / (x1 - x2).length_squared()
    //     * (x1 - x2);
    // let w2 = (2.0 * m1) / (m1 + m2) * (v2.0 - v1.0).dot(x2 - x1)
    //     / (x2 - x1).length_squared()
    //     * (x2 - x1);

    // Impulse model:
    // https://en.wikipedia.org/wiki/Collision_response

    let x1 = t1.translation.truncate();
    let x2 = t2.translation.truncate();

    // Use the x scaling as the diameter
    let r1 = t1.scale.x / 2.0;
    let r2 = t2.scale.x / 2.0;

    let distance = x1.distance(x2);

    let collision_normal = (x2 - x1).normalize();
    assert!(
        !collision_normal.x.is_nan(),
        "Found NaN in collision_normal.x"
    );
    assert!(
        !collision_normal.y.is_nan(),
        "Found NaN in collision_normal.y"
    );

    // resolve overlap
    let overlap = (r1 + r2) - distance;
    t1.translation -= (overlap / 2.0) * collision_normal.extend(0.0);
    t2.translation += (overlap / 2.0) * collision_normal.extend(0.0);

    let relative_velocity = (v2.0 - v1.0).dot(collision_normal);
    if relative_velocity > 0.0 {
        // Already moving apart
        return;
    }

    let e = 0.5; // Coefficient of restitution
    let inverse_mass_sum = (1.0 / m1.0) + (1.0 / m2.0);
    assert!(m1.0 > 0.0, "m1 is zero");
    assert!(m2.0 > 0.0, "m1 is zero");
    assert!(!inverse_mass_sum.is_nan(), "Found NaN in inverse_mass_sum");

    // Compute impulse
    let impulse = -(1.0 + e) * relative_velocity / inverse_mass_sum;
    let impulse_vector = impulse * collision_normal;
    assert!(!impulse_vector.x.is_nan(), "Found NaN in impulse_vector.x");
    assert!(!impulse_vector.y.is_nan(), "Found NaN in impulse_vector.y");

    v1.0 -= impulse_vector / m1.0;
    v2.0 += impulse_vector / m2.0;

    assert!(!v1.0.x.is_nan(), "Found NaN in v1.x");
    assert!(!v2.0.x.is_nan(), "Found NaN in v2.x");
    assert!(!t1.translation.x.is_nan(), "Found NaN in t1.x");
    assert!(!t1.translation.y.is_nan(), "Found NaN in t1.y");
    assert!(!t2.translation.x.is_nan(), "Found NaN in t2.x");
    assert!(!t2.translation.y.is_nan(), "Found NaN in t2.y");
}
