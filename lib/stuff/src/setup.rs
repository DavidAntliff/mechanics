use crate::ball::{SortedBallsCache, Stats};
use crate::benchmark::{run_benchmark, BenchmarkTargets};
use crate::cli::{Cli, Command};
use crate::fixed_frame_count_diagnostics_plugin::FixedFrameCountDiagnosticsPlugin;
use crate::stepping;
use bevy::app::{App, FixedUpdate, Update};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::{default, Fixed, PluginGroup, Time, Val, Window, WindowPlugin};
use bevy::window::PresentMode;
use bevy::DefaultPlugins;
use bevy_prng::ChaCha8Rng;
use bevy_rand::plugin::EntropyPlugin;

/// Common setup
pub fn setup(cli: &Cli) -> App {
    // Construct seed
    let x = cli.global_opts.seed.to_le_bytes();
    let mut seed = [0u8; 32];
    seed[..x.len()].copy_from_slice(&x);

    let mut app = App::new();
    app
        // Disable VSYNC
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // Turn off vsync to maximize CPU/GPU usage
                //present_mode: PresentMode::AutoNoVsync,
                present_mode: PresentMode::AutoVsync,
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
        ));

    match cli.command {
        None => (),
        Some(Command::Benchmark {
            duration: None,
            frames: None,
        }) => panic!("Benchmark command requires either a duration or a frame count"),
        Some(Command::Benchmark { duration, frames }) => {
            app.insert_resource(BenchmarkTargets {
                duration,
                fixed_frame_count: frames,
                frame_count: None,
            });
            app.add_systems(FixedUpdate, run_benchmark);
            app.add_plugins(FixedFrameCountDiagnosticsPlugin::default());
        }
    }

    // Set the physics update rate (default is 64 Hz)
    app.insert_resource(Time::<Fixed>::from_hz(cli.global_opts.physics_rate));

    app.insert_resource(Stats::default());

    app.insert_resource(SortedBallsCache::default());
    app
}
