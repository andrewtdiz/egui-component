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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InputWidth {
    Sm,
    Md,
    Lg,
}

impl InputWidth {
    pub const fn points(self) -> f32 {
        match self {
            Self::Sm => 96.0,
            Self::Md => 120.0,
            Self::Lg => 248.0,
        }
    }

    pub fn resolve(self, available_width: f32) -> f32 {
        self.points().min(available_width.max(1.0))
    }
}

pub(crate) fn resolve_input_width(
    width: f32,
    width_is_custom: bool,
    width_preset: Option<InputWidth>,
    available_width: f32,
) -> f32 {
    if width_is_custom {
        width.max(1.0)
    } else if let Some(width_preset) = width_preset {
        width_preset.resolve(available_width)
    } else {
        width.max(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_input_width, InputWidth};

    #[test]
    fn input_width_presets_use_expected_points() {
        assert_eq!(InputWidth::Sm.resolve(400.0), 96.0);
        assert_eq!(InputWidth::Md.resolve(400.0), 120.0);
        assert_eq!(InputWidth::Lg.resolve(400.0), 248.0);
    }

    #[test]
    fn input_width_presets_clamp_to_narrow_panels() {
        assert_eq!(InputWidth::Sm.resolve(72.0), 72.0);
        assert_eq!(InputWidth::Md.resolve(88.0), 88.0);
        assert_eq!(InputWidth::Lg.resolve(180.0), 180.0);
    }

    #[test]
    fn explicit_width_overrides_width_preset() {
        assert_eq!(
            resolve_input_width(212.0, true, Some(InputWidth::Sm), 80.0),
            212.0
        );
    }
}
