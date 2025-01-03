use bevy::app::{FixedLast, Plugin};
use bevy::diagnostic::{
    Diagnostic, DiagnosticPath, Diagnostics, DiagnosticsStore, RegisterDiagnostic,
};
use bevy::prelude::Res;

/// Adds FixedUpdate "frame count" diagnostic to an App.
///
/// # See also
///
/// [`LogDiagnosticsPlugin`](bevy_diagnostic::LogDiagnosticsPlugin) to output diagnostics to the console.
/// [`FrameTimeDiagnosticsPlugin`](bevy_diagnostic::FrameTimeDiagnosticsPlugin) to keep "frame time", "fps" and "frame count" stats.
#[derive(Default)]
pub struct FixedFrameCountDiagnosticsPlugin;

impl Plugin for FixedFrameCountDiagnosticsPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.register_diagnostic(Diagnostic::new(Self::FRAME_COUNT).with_smoothing_factor(0.0))
            .add_systems(FixedLast, Self::diagnostic_system);
    }
}

impl FixedFrameCountDiagnosticsPlugin {
    pub const FRAME_COUNT: DiagnosticPath = DiagnosticPath::const_new("fixed_frame_count");

    pub fn diagnostic_system(
        mut diagnostics: Diagnostics,
        diagnostics_store: Res<DiagnosticsStore>,
    ) {
        if let Some(frame_count) = diagnostics_store
            .get(&FixedFrameCountDiagnosticsPlugin::FRAME_COUNT)
            .and_then(|diagnostic| diagnostic.value())
        {
            diagnostics.add_measurement(&Self::FRAME_COUNT, || frame_count + 1f64);
        } else {
            diagnostics.add_measurement(&Self::FRAME_COUNT, || 0f64);
        }
    }
}
