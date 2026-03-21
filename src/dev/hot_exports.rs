use crate::dev::runtime::draw_showcase_surface_with_runtime;
use crate::dev::showcase::ComponentShowcaseState;

#[unsafe(no_mangle)]
pub fn hot_showcase_default_snapshot() -> String {
    serialize_showcase_snapshot(&ComponentShowcaseState::default())
}

#[unsafe(no_mangle)]
pub fn hot_showcase_frame(
    context: &egui::Context,
    snapshot_json: &str,
    reload_generation: u64,
) -> String {
    let runtime = crate::theme::ThemeRuntime::default();
    let mut state = deserialize_showcase_snapshot(snapshot_json);
    draw_showcase_surface_with_runtime(context, &mut state, reload_generation, runtime);
    serialize_showcase_snapshot(&state)
}

fn deserialize_showcase_snapshot(snapshot_json: &str) -> ComponentShowcaseState {
    serde_json::from_str(snapshot_json).unwrap_or_default()
}

fn serialize_showcase_snapshot(state: &ComponentShowcaseState) -> String {
    serde_json::to_string(state).unwrap_or_else(|_| "{}".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{hot_showcase_default_snapshot, hot_showcase_frame};
    use egui::{Context, RawInput};

    #[test]
    fn hot_showcase_snapshot_round_trip_renders() {
        let context = Context::default();
        crate::theme::install_context_resources(&context);
        let mut snapshot = hot_showcase_default_snapshot();
        let _ = context.run(RawInput::default(), |ctx| {
            snapshot = hot_showcase_frame(ctx, snapshot.as_str(), 0);
        });
        assert!(serde_json::from_str::<serde_json::Value>(snapshot.as_str()).is_ok());
    }

    #[test]
    fn invalid_snapshot_falls_back_to_default() {
        let context = Context::default();
        crate::theme::install_context_resources(&context);
        let mut snapshot = String::new();
        let _ = context.run(RawInput::default(), |ctx| {
            snapshot = hot_showcase_frame(ctx, "{not valid json", 2);
        });
        assert!(serde_json::from_str::<serde_json::Value>(snapshot.as_str()).is_ok());
    }
}
