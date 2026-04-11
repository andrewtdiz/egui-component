use std::collections::BTreeMap;

use egui_component::contract::NodeId;

const DEFAULT_DURATION_SECS: f32 = 0.3;

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize, serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum MotionProperty {
    Opacity,
    X,
    Y,
    Scale,
    ScaleX,
    ScaleY,
    Rotate,
    Width,
    Height,
    Gap,
    PaddingX,
    PaddingY,
    CornerRadius,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct MotionValues(pub BTreeMap<MotionProperty, f32>);

impl MotionValues {
    pub fn get(&self, property: MotionProperty) -> Option<f32> {
        self.0.get(&property).copied()
    }

    fn interpolate(from: &Self, to: &Self, progress: f32) -> Self {
        let mut values = BTreeMap::new();
        for (property, target) in &to.0 {
            let start = from.get(*property).unwrap_or(*target);
            values.insert(*property, start + (*target - start) * progress);
        }
        Self(values)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MotionSpec {
    #[serde(default)]
    pub initial: Option<MotionValues>,
    #[serde(default)]
    pub animate: MotionValues,
    #[serde(default)]
    pub transition: MotionTransition,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MotionTransition {
    #[serde(default = "default_duration_secs")]
    pub duration: f32,
    #[serde(default)]
    pub delay: f32,
    #[serde(default)]
    pub ease: MotionEase,
}

impl Default for MotionTransition {
    fn default() -> Self {
        Self {
            duration: DEFAULT_DURATION_SECS,
            delay: 0.0,
            ease: MotionEase::EaseOut,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionEase {
    Linear,
    EaseIn,
    #[default]
    EaseOut,
    EaseInOut,
}

impl MotionEase {
    fn sample(self, progress: f32) -> f32 {
        match self {
            Self::Linear => progress,
            Self::EaseIn => progress * progress,
            Self::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
            Self::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionAnimation {
    from: MotionValues,
    target: MotionValues,
    resolved: MotionValues,
    transition: MotionTransition,
    started_at_secs: Option<f64>,
    finished: bool,
}

impl MotionAnimation {
    pub fn from_spec(spec: MotionSpec, previous: Option<&Self>) -> Self {
        let from = previous
            .map(|animation| animation.resolved.clone())
            .or_else(|| spec.initial.clone())
            .unwrap_or_else(|| spec.animate.clone());
        let finished = from == spec.animate;

        Self {
            resolved: from.clone(),
            from,
            target: spec.animate,
            transition: spec.transition,
            started_at_secs: None,
            finished,
        }
    }

    pub fn tick(&mut self, now_secs: f64) -> bool {
        if self.finished {
            return false;
        }

        let started_at_secs = *self.started_at_secs.get_or_insert(now_secs);
        let elapsed_secs = (now_secs - started_at_secs).max(0.0) as f32;
        if elapsed_secs < self.transition.delay {
            return false;
        }

        let active_secs = elapsed_secs - self.transition.delay;
        let progress = if self.transition.duration <= f32::EPSILON {
            1.0
        } else {
            (active_secs / self.transition.duration).clamp(0.0, 1.0)
        };
        let eased = self.transition.ease.sample(progress);
        let mut next = MotionValues::interpolate(&self.from, &self.target, eased);
        if progress >= 1.0 {
            next = self.target.clone();
            self.finished = true;
        }

        let changed = self.resolved != next;
        self.resolved = next;
        changed
    }

    pub fn values(&self) -> &MotionValues {
        &self.resolved
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct MotionFrame {
    pub values: BTreeMap<NodeId, MotionValues>,
    pub active: bool,
}

impl MotionFrame {
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MotionTickResult {
    pub changed: bool,
    pub frame: MotionFrame,
}

fn default_duration_secs() -> f32 {
    DEFAULT_DURATION_SECS
}

#[cfg(test)]
mod tests {
    use super::{MotionAnimation, MotionEase, MotionProperty, MotionSpec, MotionTransition};
    use serde_json::json;

    #[test]
    fn tweens_numeric_motion_values() {
        let spec: MotionSpec = serde_json::from_value(json!({
            "initial": { "opacity": 0.0, "x": 0.0 },
            "animate": { "opacity": 1.0, "x": 10.0 },
            "transition": { "duration": 1.0, "ease": "linear" }
        }))
        .expect("motion spec should deserialize");
        let mut animation = MotionAnimation::from_spec(spec, None);

        assert_eq!(animation.values().get(MotionProperty::Opacity), Some(0.0));
        assert!(!animation.tick(0.0));
        assert!(animation.tick(0.5));
        assert_eq!(animation.values().get(MotionProperty::Opacity), Some(0.5));
        assert_eq!(animation.values().get(MotionProperty::X), Some(5.0));
        assert!(animation.tick(1.0));
        assert_eq!(animation.values().get(MotionProperty::Opacity), Some(1.0));
        assert!(animation.is_finished());
    }

    #[test]
    fn defaults_to_ease_out_transition() {
        let transition = MotionTransition::default();

        assert_eq!(transition.duration, 0.3);
        assert_eq!(transition.delay, 0.0);
        assert_eq!(transition.ease, MotionEase::EaseOut);
    }
}
