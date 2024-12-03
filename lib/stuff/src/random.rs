use bevy::prelude::*;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::*;
use rand_core::RngCore;

pub fn random_float(rng: &mut ResMut<GlobalEntropy<ChaCha8Rng>>) -> f32 {
    // Generate a u32 and normalize it to a floating-point value in [0.0, 1.0]
    let random_u32 = rng.next_u32();
    random_u32 as f32 / u32::MAX as f32
}
