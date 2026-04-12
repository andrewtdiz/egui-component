use crate::theme::ColorRole;

pub(crate) type Color = egui::Color32;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ColorAsk {
    Fill,
    FillHover,
    Text,
    TextHover,
    Border,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ThemeStyle {
    Content,
    Role(ColorRole),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct ThemeColorRef {
    pub style: ThemeStyle,
    pub ask: ColorAsk,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ColorRef {
    PalettePacked(u32),
    Theme(ThemeColorRef),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum UiRuntimeBackground {
    Solid(ColorRef),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum JustifyContent {
    Start,
    Center,
    End,
    Between,
    Around,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum AlignItems {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum AlignContent {
    Start,
    Center,
    End,
    Between,
    Around,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum AlignSelf {
    Auto,
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FlexBasis {
    Auto,
    Full,
    Pixels(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Width {
    Full,
    Pixels(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Height {
    Full,
    Pixels(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Inset {
    Pixels(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Position {
    Absolute,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum AspectDominantAxis {
    Width,
    Height,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ClipStrategy {
    Contents,
    Rect,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TextAlignY {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TextSize {
    Xs,
    Sm,
    Base,
    Lg,
    Xl,
    X2l,
    X3l,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TextUnit {
    Pixels(f32),
    FontPercent(f32),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum FontFamily {
    Sans,
    Serif,
    Mono,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum FontWeight {
    Regular,
    Medium,
    Semibold,
    Bold,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Cursor {
    Arrow,
    Hand,
    Ibeam,
    ArrowAll,
    Wait,
    WaitArrow,
    Crosshair,
    Bad,
    Hidden,
    ArrowWE,
    ArrowNS,
    ArrowNESW,
    ArrowNWSE,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Animation {
    Spin,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum EasingStyle {
    Linear,
    Sine,
    Quad,
    Cubic,
    Quart,
    Quint,
    Expo,
    Circ,
    Back,
    Elastic,
    Bounce,
}

impl Default for EasingStyle {
    fn default() -> Self {
        Self::Linear
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum EasingDirection {
    In,
    Out,
    InOut,
}

impl Default for EasingDirection {
    fn default() -> Self {
        Self::InOut
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct TransitionProps {
    pub layout: bool,
    pub transform: bool,
    pub colors: bool,
    pub opacity: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct TransitionConfig {
    pub enabled: bool,
    pub props: TransitionProps,
    pub duration_us: i32,
    pub easing_style: EasingStyle,
    pub easing_dir: EasingDirection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PaddingValue {
    Pixels(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum SideTarget {
    All,
    Horizontal,
    Vertical,
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SideValues<T: Copy> {
    pub top: Option<T>,
    pub right: Option<T>,
    pub bottom: Option<T>,
    pub left: Option<T>,
}

impl<T: Copy> Default for SideValues<T> {
    fn default() -> Self {
        Self {
            top: None,
            right: None,
            bottom: None,
            left: None,
        }
    }
}

impl<T: Copy> SideValues<T> {
    pub fn any(&self) -> bool {
        self.top.is_some() || self.right.is_some() || self.bottom.is_some() || self.left.is_some()
    }

    pub fn set(&mut self, target: SideTarget, value: T) {
        match target {
            SideTarget::All => {
                self.top = Some(value);
                self.right = Some(value);
                self.bottom = Some(value);
                self.left = Some(value);
            }
            SideTarget::Horizontal => {
                self.right = Some(value);
                self.left = Some(value);
            }
            SideTarget::Vertical => {
                self.top = Some(value);
                self.bottom = Some(value);
            }
            SideTarget::Top => self.top = Some(value),
            SideTarget::Right => self.right = Some(value),
            SideTarget::Bottom => self.bottom = Some(value),
            SideTarget::Left => self.left = Some(value),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct Translate {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TextShadow {
    pub x: f32,
    pub y: f32,
    pub opacity: f32,
    pub color: ColorRef,
}

impl Default for TextShadow {
    fn default() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            opacity: 1.0,
            color: ColorRef::PalettePacked(pack_rgba(0, 0, 0, 255)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Spec {
    pub hidden: bool,
    pub is_flex: bool,
    pub is_grid: bool,
    pub direction: Option<Direction>,
    pub justify: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub align_content: Option<AlignContent>,
    pub align_self: Option<AlignSelf>,
    pub flex_wrap: Option<FlexWrap>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<FlexBasis>,
    pub width: Option<Width>,
    pub height: Option<Height>,
    pub aspect_ratio: Option<f32>,
    pub aspect_dominant_axis: Option<AspectDominantAxis>,
    pub grid_cols: Option<usize>,
    pub grid_rows: Option<usize>,
    pub gap_col: Option<f32>,
    pub gap_row: Option<f32>,
    pub margin: SideValues<f32>,
    pub padding: SideValues<PaddingValue>,
    pub border: SideValues<f32>,
    pub border_color: Option<ColorRef>,
    pub corner_radius: Option<f32>,
    pub top: Option<Inset>,
    pub right: Option<Inset>,
    pub bottom: Option<Inset>,
    pub left: Option<Inset>,
    pub position: Option<Position>,
    pub layout_anchor: Option<[f32; 2]>,
    pub clip_children: Option<bool>,
    pub clip_strategy: Option<ClipStrategy>,
    pub scroll_x: bool,
    pub scroll_y: bool,
    pub text_align: Option<TextAlign>,
    pub text_align_y: Option<TextAlignY>,
    pub text_wrap: bool,
    pub break_words: bool,
    pub text_size: Option<TextSize>,
    pub font_scale: Option<f32>,
    pub font_family: Option<FontFamily>,
    pub font_weight: Option<FontWeight>,
    pub font_id: Option<String>,
    pub text_tracking: Option<TextUnit>,
    pub text_leading: Option<TextUnit>,
    pub text_max_lines: Option<usize>,
    pub text_outline_color: Option<ColorRef>,
    pub text_outline_thickness: Option<TextUnit>,
    pub text_shadow: Option<TextShadow>,
    pub text: Option<ColorRef>,
    pub background: Option<UiRuntimeBackground>,
    pub opacity: Option<f32>,
    pub scale: Option<f32>,
    pub translate: Option<Translate>,
    pub rotation_degrees: Option<f32>,
    pub z_index: i16,
    pub cursor: Option<Cursor>,
    pub animation: Option<Animation>,
    pub transition: TransitionConfig,
    pub group: bool,
    pub hover_background: Option<UiRuntimeBackground>,
    pub hover_text: Option<ColorRef>,
    pub hover_text_outline_color: Option<ColorRef>,
    pub hover_text_outline_thickness: Option<TextUnit>,
    pub hover_opacity: Option<f32>,
    pub hover_scale: Option<f32>,
    pub hover_cursor: Option<Cursor>,
    pub hover_margin: SideValues<f32>,
    pub hover_padding: SideValues<PaddingValue>,
    pub hover_border: SideValues<f32>,
    pub hover_border_color: Option<ColorRef>,
    pub group_hover_background: Option<UiRuntimeBackground>,
    pub group_hover_text: Option<ColorRef>,
    pub group_hover_text_outline_color: Option<ColorRef>,
    pub group_hover_text_outline_thickness: Option<TextUnit>,
    pub group_hover_opacity: Option<f32>,
    pub group_hover_scale: Option<f32>,
    pub group_hover_cursor: Option<Cursor>,
    pub group_hover_margin: SideValues<f32>,
    pub group_hover_padding: SideValues<PaddingValue>,
    pub group_hover_border: SideValues<f32>,
    pub group_hover_border_color: Option<ColorRef>,
}

impl Default for Spec {
    fn default() -> Self {
        Self {
            hidden: false,
            is_flex: false,
            is_grid: false,
            direction: None,
            justify: None,
            align_items: None,
            align_content: None,
            align_self: None,
            flex_wrap: None,
            flex_grow: None,
            flex_shrink: None,
            flex_basis: None,
            width: None,
            height: None,
            aspect_ratio: None,
            aspect_dominant_axis: None,
            grid_cols: None,
            grid_rows: None,
            gap_col: None,
            gap_row: None,
            margin: SideValues::default(),
            padding: SideValues::default(),
            border: SideValues::default(),
            border_color: None,
            corner_radius: None,
            top: None,
            right: None,
            bottom: None,
            left: None,
            position: None,
            layout_anchor: None,
            clip_children: None,
            clip_strategy: None,
            scroll_x: false,
            scroll_y: false,
            text_align: None,
            text_align_y: None,
            text_wrap: true,
            break_words: false,
            text_size: None,
            font_scale: None,
            font_family: None,
            font_weight: None,
            font_id: None,
            text_tracking: None,
            text_leading: None,
            text_max_lines: None,
            text_outline_color: None,
            text_outline_thickness: None,
            text_shadow: None,
            text: None,
            background: None,
            opacity: None,
            scale: None,
            translate: None,
            rotation_degrees: None,
            z_index: 0,
            cursor: None,
            animation: None,
            transition: TransitionConfig::default(),
            group: false,
            hover_background: None,
            hover_text: None,
            hover_text_outline_color: None,
            hover_text_outline_thickness: None,
            hover_opacity: None,
            hover_scale: None,
            hover_cursor: None,
            hover_margin: SideValues::default(),
            hover_padding: SideValues::default(),
            hover_border: SideValues::default(),
            hover_border_color: None,
            group_hover_background: None,
            group_hover_text: None,
            group_hover_text_outline_color: None,
            group_hover_text_outline_thickness: None,
            group_hover_opacity: None,
            group_hover_scale: None,
            group_hover_cursor: None,
            group_hover_margin: SideValues::default(),
            group_hover_padding: SideValues::default(),
            group_hover_border: SideValues::default(),
            group_hover_border_color: None,
        }
    }
}

pub(crate) const fn pack_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> u32 {
    ((red as u32) << 24) | ((green as u32) << 16) | ((blue as u32) << 8) | alpha as u32
}

pub(crate) fn unpack_rgba(value: u32) -> Color {
    Color::from_rgba_unmultiplied(
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    )
}
