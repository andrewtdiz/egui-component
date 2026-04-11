#[path = "catalog.rs"]
mod catalog_defs;
pub mod components;
pub mod contract;
#[path = "example_apps/mod.rs"]
mod demo_apps_internal;
mod internal_taffy;
pub mod layout;
pub mod icons {
    pub use crate::ui::icons::{image, setup, svg_source};
}
pub mod primitives;
pub mod theme;
pub mod ui;

/// Support modules used by the repository's local examples and demo tooling.
///
/// This is not part of the main component-library surface.
pub mod demos {
    pub use crate::demo_apps_internal::{contract_demo, showcase};
}

#[doc(hidden)]
#[deprecated(
    note = "use egui_component::demos::* for local demo support, or run the example binaries directly"
)]
pub mod example_apps {
    pub use crate::demos::*;
}

#[doc(hidden)]
#[deprecated(
    note = "use the crate root re-exports like ComponentKind and component_definitions() instead"
)]
pub mod catalog {
    pub use crate::catalog_defs::{
        component_definitions, component_definitions_by_group, parse_component_kind,
        supported_component_ids_csv, ComponentDefinition, ComponentGroup, ComponentKind,
    };
}

pub(crate) use internal_taffy::{
    setup_tui_visuals, tid, AsTuiBuilder, TaffyContainerUi, Tui, TuiBuilder, TuiBuilderLogic,
    TuiBuilderParamsAccess, TuiContainerResponse, TuiId, TuiInnerResponse, TuiWidget,
};

pub use catalog_defs::{
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
