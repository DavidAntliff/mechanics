use crate::ball::Stats;
use crate::fixed_frame_count_diagnostics_plugin::FixedFrameCountDiagnosticsPlugin;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::{Real, Res, Resource, Time};

#[derive(Resource)]
pub struct BenchmarkTargets {
    pub duration: Option<f64>,
    pub fixed_frame_count: Option<f64>,
    pub frame_count: Option<f64>,
}

// Use a variation of the DiagnosticsStore resource that records
// the number of FixedUpdate frames, not Update frames, so that we can
// benchmark the time to render a fixed number of physics frames.
pub fn run_benchmark(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time<Real>>,
    targets: Res<BenchmarkTargets>,
    stats: Res<Stats>,
) {
    if let Some(fixed_frame_count) = diagnostics
        .get(&FixedFrameCountDiagnosticsPlugin::FRAME_COUNT)
        .and_then(|diagnostic| diagnostic.value())
    {
        if let Some(frame_count) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FRAME_COUNT)
            .and_then(|diagnostic| diagnostic.value())
        {
            let duration = time.elapsed_seconds_f64();
            let mut exit = false;

            match *targets {
                BenchmarkTargets {
                    duration: Some(duration_target),
                    fixed_frame_count: None,
                    frame_count: None,
                } => {
                    if duration >= duration_target {
                        exit = true;
                    }
                }
                BenchmarkTargets {
                    duration: None,
                    fixed_frame_count: Some(frames_target),
                    frame_count: None,
                } => {
                    if fixed_frame_count >= frames_target {
                        exit = true;
                    }
                }
                _ => {}
            }

            if exit {
                println!(
                    "Stopping the app at {:.2} seconds after {:.0} fixed frames, {:.2} fxd-fps, {:.2} fps",
                    duration,
                    fixed_frame_count,
                    fixed_frame_count / duration,
                    frame_count / duration,
                );
                println!("Number of collisions: {}", stats.num_collisions);
                std::process::exit(0);
            }
        } else {
            //println!("FrameTimeDiagnosticsPlugin::FRAME_COUNT not found");
        }
    } else {
        //println!("FixedFrameCountDiagnosticsPlugin::FRAME_COUNT not found");
    }
}
