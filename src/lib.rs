pub mod catalog;
pub mod components;
pub mod contract;
#[doc(hidden)]
pub mod example_apps;
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

pub use catalog::{
    component_definitions, component_definitions_by_group, parse_component_kind,
    supported_component_ids_csv, ComponentDefinition, ComponentGroup, ComponentKind,
};
pub use theme::{
    BaseColor, ColorRole, OklchColor, RadiusRole, ShadowRole, ThemeMode, ThemePalette,
    ThemeShadows, ThemeSpec,
};

pub mod prelude {
    pub use crate::components::*;
    pub use crate::theme::{
        BaseColor, ColorRole, OklchColor, RadiusRole, ShadowRole, ThemeMode, ThemePalette,
        ThemeShadows, ThemeSpec,
    };
}
