#[path = "catalog.rs"]
mod catalog_defs;
pub mod contract;
#[path = "components/mod.rs"]
pub(crate) mod runtime_components;

#[deprecated(
    note = "direct Rust component authoring is deprecated; author UI in JSX/TSX and render it through egui_component::contract::*"
)]
pub mod components {
    pub use crate::runtime_components::*;
}
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
#[deprecated(
    note = "host-authored Rust demos are deprecated; use the JSX runtime examples under examples/runtime-jsx instead"
)]
pub mod demos {
    pub use crate::demo_apps_internal::{contract_demo, showcase};
}

#[doc(hidden)]
#[deprecated(
    note = "use the JSX runtime examples directly; the host-authored demo shims remain only for legacy compatibility"
)]
pub mod example_apps {
    pub use crate::demo_apps_internal::{contract_demo, showcase};
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
