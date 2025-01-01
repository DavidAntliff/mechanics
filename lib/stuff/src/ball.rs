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

pub fn ball_collision_system(
    mut query: Query<(&mut Transform, &mut Velocity, &Mass), With<Ball>>,
    //mut stepping: ResMut<Stepping>,
    //mut collision_events: EventWriter<CollisionEvent>,
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
            //stepping.enable();

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

            let collision_normal = (x2 - x1).normalize();
            //dbg!(collision_normal);

            // resolve overlap
            let overlap = (r1 + r2) - distance;
            t1.translation -= (overlap / 2.0) * collision_normal.extend(0.0);
            t2.translation += (overlap / 2.0) * collision_normal.extend(0.0);

            // let v_normal = (v1.0 - v2.0).dot(x1 - x2);
            // if v_normal > 0.0 {
            //     // Already moving apart
            //     continue;
            // }

            let relative_velocity = (v2.0 - v1.0).dot(collision_normal);
            //dbg!(relative_velocity);
            if relative_velocity > 0.0 {
                // Already moving apart
                continue;
            }

            let e = 0.5; // Coefficient of restitution
            let inverse_mass_sum = (1.0 / m1.0) + (1.0 / m2.0);

            // Compute impulse
            let impulse = -(1.0 + e) * relative_velocity / inverse_mass_sum;
            let impulse_vector = impulse * collision_normal;
            //dbg!(impulse_vector);

            v1.0 -= impulse_vector / m1.0;
            v2.0 += impulse_vector / m2.0;

            // let combined_mass = m1.0 + m2.0;
            // let w1 = (2.0 * m2.0) / combined_mass * e * relative_velocity * collision_normal;
            // let w2 = (2.0 * m1.0) / combined_mass * e * relative_velocity * collision_normal;
            //
            // v1.0 -= w1;
            // v2.0 += w2;

            // let w1 =
            //     (2.0 * m2.0) / combined_mass * v_normal / (x1 - x2).length_squared() * (x1 - x2);
            // let w2 =
            //     (2.0 * m1.0) / combined_mass * v_normal / (x2 - x1).length_squared() * (x2 - x1);
            //
            // v1.0 -= w1;
            // v2.0 -= w2;
        }
    }
}
