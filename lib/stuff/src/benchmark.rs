use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::{Real, Res, Resource, Time};

#[derive(Resource)]
pub struct BenchmarkTargets {
    pub duration: Option<f64>,
    pub frame_count: Option<f64>,
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
