pub mod contract;
#[path = "components/mod.rs"]
pub(crate) mod runtime_components;

#[deprecated(
    note = "direct Rust component authoring is deprecated; author UI in JSX/TSX and render it through egui_component::contract::*"
)]
pub mod components {
    pub use crate::runtime_components::*;
}
mod internal_taffy;
pub mod layout;
pub mod icons {
    pub use crate::ui::icons::{image, setup, svg_source};
}
pub mod primitives;
pub mod theme;
pub mod ui;

pub(crate) use internal_taffy::{
    setup_tui_visuals, tid, AsTuiBuilder, TaffyContainerUi, Tui, TuiBuilder, TuiBuilderLogic,
    TuiBuilderParamsAccess, TuiContainerResponse, TuiId, TuiInnerResponse, TuiWidget,
};
pub use theme::{
    BaseColor, ColorRole, OklchColor, RadiusRole, ShadowRole, ThemeMode, ThemePalette,
    ThemeShadows, ThemeSpec,
};

#[deprecated(
    note = "direct Rust component authoring through egui_component::prelude::* and ui.components() is deprecated; author UI in JSX/TSX and render it through egui_component::contract::*"
)]
pub mod prelude {
    pub use crate::primitives::ScrollAreaExt;
    pub use crate::runtime_components::*;
    pub use crate::theme::{
        BaseColor, ColorRole, OklchColor, RadiusRole, ShadowRole, ThemeMode, ThemePalette,
        ThemeShadows, ThemeSpec,
    };
}
