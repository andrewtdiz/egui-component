#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ControlSize {
    Sm,
    Md,
}

impl ControlSize {
    pub const fn is_small(self) -> bool {
        matches!(self, Self::Sm)
    }

    pub const fn min_interact_height(self) -> f32 {
        match self {
            Self::Sm => 28.0,
            Self::Md => 34.0,
        }
    }
}
