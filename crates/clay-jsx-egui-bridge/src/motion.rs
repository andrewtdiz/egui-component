//! Retained motion values for the egui JSX bridge.
//!
//! Motion data is stored separately from the retained contract tree so motion
//! retargets and frame ticks can update animation state without implying a
//! structural contract-tree change.

use std::collections::BTreeMap;

use clay_jsx_runtime::contract::NodeId;

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

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct MotionFrame {
    pub values: BTreeMap<NodeId, MotionValues>,
    pub active: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MotionTickResult {
    pub changed: bool,
    pub frame: MotionFrame,
}

fn default_duration_secs() -> f32 {
    DEFAULT_DURATION_SECS
}
