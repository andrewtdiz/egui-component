use std::collections::BTreeMap;

use clay_jsx_runtime::contract::NodeId;

const DEFAULT_DURATION_SECS: f32 = 0.3;

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize, serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
#[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
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
#[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
pub struct MotionValues(pub BTreeMap<MotionProperty, f32>);

impl MotionValues {
    #[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
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
#[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
pub struct MotionSpec {
    #[serde(default)]
    pub initial: Option<MotionValues>,
    #[serde(default)]
    pub animate: MotionValues,
    #[serde(default)]
    pub transition: MotionTransition,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
#[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
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
#[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
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
#[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
pub struct MotionAnimation {
    from: MotionValues,
    target: MotionValues,
    resolved: MotionValues,
    transition: MotionTransition,
    started_at_secs: Option<f64>,
    finished: bool,
}

impl MotionAnimation {
    #[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
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

    #[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
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

    #[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
    pub fn values(&self) -> &MotionValues {
        &self.resolved
    }

    #[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
#[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
pub struct MotionFrame {
    pub values: BTreeMap<NodeId, MotionValues>,
    pub active: bool,
}

impl MotionFrame {
    #[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
#[deprecated(note = "motion driver is being reworked; API surface preserved, implementation is going away")]
pub struct MotionTickResult {
    pub changed: bool,
    pub frame: MotionFrame,
}

fn default_duration_secs() -> f32 {
    DEFAULT_DURATION_SECS
}
