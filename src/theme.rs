mod presets;

use crate::{icons, ui::style};
use egui::{Color32, Context, Id, Shadow, Ui};
use std::fmt;

const DESTRUCTIVE_FOREGROUND: OklchColor = OklchColor::new(0.985, 0.0, 0.0);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
}

impl ThemeMode {
    pub const fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum BaseColor {
    #[default]
    Neutral,
    Stone,
    Zinc,
    Mauve,
    Olive,
    Mist,
    Taupe,
}

impl BaseColor {
    pub const ALL: [Self; 7] = [
        Self::Neutral,
        Self::Stone,
        Self::Zinc,
        Self::Mauve,
        Self::Olive,
        Self::Mist,
        Self::Taupe,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Neutral => "Neutral",
            Self::Stone => "Stone",
            Self::Zinc => "Zinc",
            Self::Mauve => "Mauve",
            Self::Olive => "Olive",
            Self::Mist => "Mist",
            Self::Taupe => "Taupe",
        }
    }

    pub const fn theme(self) -> ThemeSpec {
        match self {
            Self::Neutral => presets::neutral_theme(),
            Self::Stone => presets::stone_theme(),
            Self::Zinc => presets::zinc_theme(),
            Self::Mauve => presets::mauve_theme(),
            Self::Olive => presets::olive_theme(),
            Self::Mist => presets::mist_theme(),
            Self::Taupe => presets::taupe_theme(),
        }
    }
}

impl fmt::Display for BaseColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OklchColor {
    pub lightness: f32,
    pub chroma: f32,
    pub hue: f32,
    pub alpha: f32,
}

impl OklchColor {
    pub const fn new(lightness: f32, chroma: f32, hue: f32) -> Self {
        Self {
            lightness,
            chroma,
            hue,
            alpha: 1.0,
        }
    }

    pub const fn with_alpha(lightness: f32, chroma: f32, hue: f32, alpha: f32) -> Self {
        Self {
            lightness,
            chroma,
            hue,
            alpha,
        }
    }

    pub fn to_color32(self) -> Color32 {
        let hue = self.hue.to_radians();
        let a = self.chroma * hue.cos();
        let b = self.chroma * hue.sin();

        let l_component = cube(self.lightness + 0.396_337_78 * a + 0.215_803_76 * b);
        let m_component = cube(self.lightness - 0.105_561_346 * a - 0.063_854_17 * b);
        let s_component = cube(self.lightness - 0.089_484_18 * a - 1.291_485_5 * b);

        let red_linear =
            4.076_741_7 * l_component - 3.307_711_6 * m_component + 0.230_969_94 * s_component;
        let green_linear =
            -1.268_438 * l_component + 2.609_757_4 * m_component - 0.341_319_38 * s_component;
        let blue_linear =
            -0.004_196_086_3 * l_component - 0.703_418_6 * m_component + 1.707_614_7 * s_component;

        Color32::from_rgba_unmultiplied(
            linear_to_srgb_channel(red_linear),
            linear_to_srgb_channel(green_linear),
            linear_to_srgb_channel(blue_linear),
            alpha_to_u8(self.alpha),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemePalette {
    pub background: OklchColor,
    pub foreground: OklchColor,
    pub card: OklchColor,
    pub card_foreground: OklchColor,
    pub popover: OklchColor,
    pub popover_foreground: OklchColor,
    pub primary: OklchColor,
    pub primary_foreground: OklchColor,
    pub secondary: OklchColor,
    pub secondary_foreground: OklchColor,
    pub muted: OklchColor,
    pub muted_foreground: OklchColor,
    pub accent: OklchColor,
    pub accent_foreground: OklchColor,
    pub destructive: OklchColor,
    pub destructive_foreground: OklchColor,
    pub border: OklchColor,
    pub input: OklchColor,
    pub ring: OklchColor,
    pub chart_1: OklchColor,
    pub chart_2: OklchColor,
    pub chart_3: OklchColor,
    pub chart_4: OklchColor,
    pub chart_5: OklchColor,
    pub sidebar: OklchColor,
    pub sidebar_foreground: OklchColor,
    pub sidebar_primary: OklchColor,
    pub sidebar_primary_foreground: OklchColor,
    pub sidebar_accent: OklchColor,
    pub sidebar_accent_foreground: OklchColor,
    pub sidebar_border: OklchColor,
    pub sidebar_ring: OklchColor,
}

impl ThemePalette {
    pub const fn color(self, role: ColorRole) -> OklchColor {
        match role {
            ColorRole::Background => self.background,
            ColorRole::Foreground => self.foreground,
            ColorRole::Card => self.card,
            ColorRole::CardForeground => self.card_foreground,
            ColorRole::Popover => self.popover,
            ColorRole::PopoverForeground => self.popover_foreground,
            ColorRole::Primary => self.primary,
            ColorRole::PrimaryForeground => self.primary_foreground,
            ColorRole::Secondary => self.secondary,
            ColorRole::SecondaryForeground => self.secondary_foreground,
            ColorRole::Muted => self.muted,
            ColorRole::MutedForeground => self.muted_foreground,
            ColorRole::Accent => self.accent,
            ColorRole::AccentForeground => self.accent_foreground,
            ColorRole::Destructive => self.destructive,
            ColorRole::DestructiveForeground => self.destructive_foreground,
            ColorRole::Border => self.border,
            ColorRole::Input => self.input,
            ColorRole::Ring => self.ring,
            ColorRole::Chart1 => self.chart_1,
            ColorRole::Chart2 => self.chart_2,
            ColorRole::Chart3 => self.chart_3,
            ColorRole::Chart4 => self.chart_4,
            ColorRole::Chart5 => self.chart_5,
            ColorRole::Sidebar => self.sidebar,
            ColorRole::SidebarForeground => self.sidebar_foreground,
            ColorRole::SidebarPrimary => self.sidebar_primary,
            ColorRole::SidebarPrimaryForeground => self.sidebar_primary_foreground,
            ColorRole::SidebarAccent => self.sidebar_accent,
            ColorRole::SidebarAccentForeground => self.sidebar_accent_foreground,
            ColorRole::SidebarBorder => self.sidebar_border,
            ColorRole::SidebarRing => self.sidebar_ring,
        }
    }

    pub fn resolved(self, role: ColorRole) -> Color32 {
        self.color(role).to_color32()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeShadows {
    pub sm: Shadow,
    pub md: Shadow,
    pub lg: Shadow,
}

impl ThemeShadows {
    pub const fn new(sm: Shadow, md: Shadow, lg: Shadow) -> Self {
        Self { sm, md, lg }
    }

    pub const fn shadow(self, role: ShadowRole) -> Shadow {
        match role {
            ShadowRole::Sm => self.sm,
            ShadowRole::Md => self.md,
            ShadowRole::Lg => self.lg,
        }
    }
}

impl Default for ThemeShadows {
    fn default() -> Self {
        default_shadows()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeSpec {
    pub radius: f32,
    pub light: ThemePalette,
    pub dark: ThemePalette,
    pub shadows: ThemeShadows,
}

impl ThemeSpec {
    pub const fn preset(base: BaseColor) -> Self {
        base.theme()
    }

    pub const fn palette(self, mode: ThemeMode) -> ThemePalette {
        if mode.is_dark() {
            self.dark
        } else {
            self.light
        }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius.max(0.0);
        self
    }
}

impl Default for ThemeSpec {
    fn default() -> Self {
        Self::preset(BaseColor::Neutral)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ColorRole {
    Background,
    Foreground,
    Card,
    CardForeground,
    Popover,
    PopoverForeground,
    Primary,
    PrimaryForeground,
    Secondary,
    SecondaryForeground,
    Muted,
    MutedForeground,
    Accent,
    AccentForeground,
    Destructive,
    DestructiveForeground,
    Border,
    Input,
    Ring,
    Chart1,
    Chart2,
    Chart3,
    Chart4,
    Chart5,
    Sidebar,
    SidebarForeground,
    SidebarPrimary,
    SidebarPrimaryForeground,
    SidebarAccent,
    SidebarAccentForeground,
    SidebarBorder,
    SidebarRing,
}

impl ColorRole {
    pub const ALL: [Self; 32] = [
        Self::Background,
        Self::Foreground,
        Self::Card,
        Self::CardForeground,
        Self::Popover,
        Self::PopoverForeground,
        Self::Primary,
        Self::PrimaryForeground,
        Self::Secondary,
        Self::SecondaryForeground,
        Self::Muted,
        Self::MutedForeground,
        Self::Accent,
        Self::AccentForeground,
        Self::Destructive,
        Self::DestructiveForeground,
        Self::Border,
        Self::Input,
        Self::Ring,
        Self::Chart1,
        Self::Chart2,
        Self::Chart3,
        Self::Chart4,
        Self::Chart5,
        Self::Sidebar,
        Self::SidebarForeground,
        Self::SidebarPrimary,
        Self::SidebarPrimaryForeground,
        Self::SidebarAccent,
        Self::SidebarAccentForeground,
        Self::SidebarBorder,
        Self::SidebarRing,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Foreground => "foreground",
            Self::Card => "card",
            Self::CardForeground => "card-foreground",
            Self::Popover => "popover",
            Self::PopoverForeground => "popover-foreground",
            Self::Primary => "primary",
            Self::PrimaryForeground => "primary-foreground",
            Self::Secondary => "secondary",
            Self::SecondaryForeground => "secondary-foreground",
            Self::Muted => "muted",
            Self::MutedForeground => "muted-foreground",
            Self::Accent => "accent",
            Self::AccentForeground => "accent-foreground",
            Self::Destructive => "destructive",
            Self::DestructiveForeground => "destructive-foreground",
            Self::Border => "border",
            Self::Input => "input",
            Self::Ring => "ring",
            Self::Chart1 => "chart-1",
            Self::Chart2 => "chart-2",
            Self::Chart3 => "chart-3",
            Self::Chart4 => "chart-4",
            Self::Chart5 => "chart-5",
            Self::Sidebar => "sidebar",
            Self::SidebarForeground => "sidebar-foreground",
            Self::SidebarPrimary => "sidebar-primary",
            Self::SidebarPrimaryForeground => "sidebar-primary-foreground",
            Self::SidebarAccent => "sidebar-accent",
            Self::SidebarAccentForeground => "sidebar-accent-foreground",
            Self::SidebarBorder => "sidebar-border",
            Self::SidebarRing => "sidebar-ring",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RadiusRole {
    Sm,
    Md,
    Lg,
    Xl,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ShadowRole {
    Sm,
    Md,
    Lg,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ThemeRuntime {
    pub spec: ThemeSpec,
    pub mode: ThemeMode,
}

impl ThemeRuntime {
    pub const fn new(spec: ThemeSpec, mode: ThemeMode) -> Self {
        Self { spec, mode }
    }

    pub const fn palette(self) -> ThemePalette {
        self.spec.palette(self.mode)
    }

    pub const fn with_mode(self, mode: ThemeMode) -> Self {
        Self {
            spec: self.spec,
            mode,
        }
    }
}

impl Default for ThemeRuntime {
    fn default() -> Self {
        Self::new(ThemeSpec::default(), ThemeMode::Dark)
    }
}

pub const fn preset(base: BaseColor) -> ThemeSpec {
    ThemeSpec::preset(base)
}

pub fn install_context_resources(context: &Context) {
    style::install_context_resources(context);
    icons::setup(context);
}

pub fn install(context: &Context, theme: ThemeSpec, mode: ThemeMode) {
    let runtime = ThemeRuntime::new(theme, mode);
    store_context_runtime(context, runtime);
    style::install(context, runtime);
    icons::setup(context);
}

pub fn set_theme(context: &Context, theme: ThemeSpec) {
    let runtime = ThemeRuntime::new(theme, load_context_runtime(context).mode);
    apply_context_runtime(context, runtime);
}

pub fn set_mode(context: &Context, mode: ThemeMode) {
    let runtime = ThemeRuntime::new(load_context_runtime(context).spec, mode);
    apply_context_runtime(context, runtime);
}

pub fn with_theme<R>(
    ui: &mut Ui,
    theme: ThemeSpec,
    mode: ThemeMode,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let runtime = ThemeRuntime::new(theme, mode);
    let previous = store_scoped_runtime(ui, runtime);
    let inner = ui
        .scope(|ui| {
            style::apply_to_ui(ui, runtime);
            add(ui)
        })
        .inner;
    restore_scoped_runtime(ui, previous);
    inner
}

pub fn color(ui: &Ui, role: ColorRole) -> Color32 {
    runtime_for_ui(ui).palette().resolved(role)
}

pub fn radius(ui: &Ui, role: RadiusRole) -> u8 {
    radius_for_spec(runtime_for_ui(ui).spec, role)
}

pub fn shadow(ui: &Ui, role: ShadowRole) -> Shadow {
    runtime_for_ui(ui).spec.shadows.shadow(role)
}

pub(crate) fn resolved_color(runtime: ThemeRuntime, role: ColorRole) -> Color32 {
    runtime.palette().resolved(role)
}

pub(crate) fn resolved_shadow(runtime: ThemeRuntime, role: ShadowRole) -> Shadow {
    runtime.spec.shadows.shadow(role)
}

pub(crate) fn resolved_radius(runtime: ThemeRuntime, role: RadiusRole) -> u8 {
    radius_for_spec(runtime.spec, role)
}

pub(crate) const fn default_shadows() -> ThemeShadows {
    ThemeShadows::new(
        Shadow {
            offset: [0, 1],
            blur: 2,
            spread: 0,
            color: Color32::from_rgba_premultiplied(0, 0, 0, 10),
        },
        Shadow {
            offset: [0, 2],
            blur: 4,
            spread: 0,
            color: Color32::from_rgba_premultiplied(0, 0, 0, 18),
        },
        Shadow {
            offset: [0, 6],
            blur: 10,
            spread: 0,
            color: Color32::from_rgba_premultiplied(0, 0, 0, 22),
        },
    )
}

pub(crate) trait ThemeUiRef {
    fn theme_ui(&self) -> &Ui;
}

impl ThemeUiRef for Ui {
    fn theme_ui(&self) -> &Ui {
        self
    }
}

impl ThemeUiRef for crate::components::ComponentUi<'_> {
    fn theme_ui(&self) -> &Ui {
        self.ui()
    }
}

pub(crate) fn runtime_for_context(context: &Context) -> ThemeRuntime {
    context
        .data(|data| data.get_temp::<ThemeRuntime>(context_theme_runtime_id()))
        .unwrap_or_default()
}

pub(crate) fn runtime_for_ui(ui: &impl ThemeUiRef) -> ThemeRuntime {
    let ui = ui.theme_ui();
    if let Some(runtime) = ui.data(|data| data.get_temp::<ThemeRuntime>(scoped_theme_runtime_id()))
    {
        runtime
    } else {
        runtime_for_context(ui.ctx())
    }
}

pub(crate) fn apply_context_runtime(context: &Context, runtime: ThemeRuntime) {
    store_context_runtime(context, runtime);
    style::set_theme_runtime(context, runtime);
}

fn load_context_runtime(context: &Context) -> ThemeRuntime {
    runtime_for_context(context)
}

fn store_context_runtime(context: &Context, runtime: ThemeRuntime) {
    context.data_mut(|data| data.insert_temp(context_theme_runtime_id(), runtime));
}

fn store_scoped_runtime(ui: &mut Ui, runtime: ThemeRuntime) -> Option<ThemeRuntime> {
    let previous = ui.data(|data| data.get_temp::<ThemeRuntime>(scoped_theme_runtime_id()));
    ui.data_mut(|data| data.insert_temp(scoped_theme_runtime_id(), runtime));
    previous
}

fn restore_scoped_runtime(ui: &mut Ui, previous: Option<ThemeRuntime>) {
    ui.data_mut(|data| {
        if let Some(previous) = previous {
            data.insert_temp(scoped_theme_runtime_id(), previous);
        } else {
            data.remove::<ThemeRuntime>(scoped_theme_runtime_id());
        }
    });
}

fn radius_for_spec(spec: ThemeSpec, role: RadiusRole) -> u8 {
    let radius = match role {
        RadiusRole::Sm => (spec.radius - 4.0).max(0.0),
        RadiusRole::Md => (spec.radius - 2.0).max(0.0),
        RadiusRole::Lg => spec.radius.max(0.0),
        RadiusRole::Xl => (spec.radius + 4.0).max(0.0),
    };
    radius.round().clamp(0.0, 255.0) as u8
}

fn cube(value: f32) -> f32 {
    value * value * value
}

fn linear_to_srgb_channel(value: f32) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    let gamma = if clamped <= 0.003_130_8 {
        clamped * 12.92
    } else {
        (1.055 * clamped.powf(1.0 / 2.4)) - 0.055
    };
    ((gamma * 255.0).round()).clamp(0.0, 255.0) as u8
}

fn alpha_to_u8(alpha: f32) -> u8 {
    ((alpha.clamp(0.0, 1.0) * 255.0).round()).clamp(0.0, 255.0) as u8
}

fn context_theme_runtime_id() -> Id {
    Id::new("egui_component::theme_runtime")
}

fn scoped_theme_runtime_id() -> Id {
    Id::new("egui_component::scoped_theme_runtime")
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{CentralPanel, RawInput};

    #[test]
    fn base_color_presets_match_upstream_slots() {
        let neutral = ThemeSpec::preset(BaseColor::Neutral);
        assert_eq!(neutral.light.background, OklchColor::new(1.0, 0.0, 0.0));
        assert_eq!(
            neutral.dark.sidebar_primary,
            OklchColor::new(0.488, 0.243, 264.376)
        );
        assert_eq!(neutral.light.destructive_foreground, DESTRUCTIVE_FOREGROUND);

        let taupe = ThemeSpec::preset(BaseColor::Taupe);
        assert_eq!(taupe.light.primary, OklchColor::new(0.214, 0.009, 43.1));
        assert_eq!(taupe.dark.sidebar_ring, OklchColor::new(0.547, 0.021, 43.1));
    }

    #[test]
    fn radius_roles_derive_from_root_radius() {
        let theme = ThemeSpec::default().with_radius(10.0);
        assert_eq!(radius_for_spec(theme, RadiusRole::Sm), 6);
        assert_eq!(radius_for_spec(theme, RadiusRole::Md), 8);
        assert_eq!(radius_for_spec(theme, RadiusRole::Lg), 10);
        assert_eq!(radius_for_spec(theme, RadiusRole::Xl), 14);
    }

    #[test]
    fn set_theme_preserves_mode_and_scoped_theme_restores() {
        let context = Context::default();
        install(
            &context,
            ThemeSpec::preset(BaseColor::Neutral),
            ThemeMode::Dark,
        );
        set_theme(&context, ThemeSpec::preset(BaseColor::Stone));

        assert_eq!(load_context_runtime(&context).mode, ThemeMode::Dark);

        let _ = context.run(RawInput::default(), |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                assert_eq!(
                    color(ui, ColorRole::Background),
                    ThemeSpec::preset(BaseColor::Stone)
                        .palette(ThemeMode::Dark)
                        .resolved(ColorRole::Background)
                );

                with_theme(
                    ui,
                    ThemeSpec::preset(BaseColor::Mauve),
                    ThemeMode::Light,
                    |ui| {
                        assert_eq!(
                            color(ui, ColorRole::Background),
                            ThemeSpec::preset(BaseColor::Mauve)
                                .palette(ThemeMode::Light)
                                .resolved(ColorRole::Background)
                        );
                    },
                );

                assert_eq!(
                    color(ui, ColorRole::Background),
                    ThemeSpec::preset(BaseColor::Stone)
                        .palette(ThemeMode::Dark)
                        .resolved(ColorRole::Background)
                );
            });
        });
    }

    #[test]
    fn oklch_conversion_preserves_alpha_and_basic_neutral_values() {
        assert_eq!(OklchColor::new(1.0, 0.0, 0.0).to_color32(), Color32::WHITE);
        assert_eq!(
            OklchColor::with_alpha(1.0, 0.0, 0.0, 0.1).to_color32().a(),
            26
        );
        assert!(OklchColor::new(0.577, 0.245, 27.325).to_color32().r() > 128);
    }
}
