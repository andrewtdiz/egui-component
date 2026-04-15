mod app;
mod metrics;
mod watch;

use std::path::PathBuf;

use egui::{Context, ViewportBuilder};
use egui_component::theme::{self, BaseColor, ThemeMode, ThemeSpec};

pub use app::RuntimeJsxApp;
pub use metrics::{request_host_repaint, ExampleHostDebugSnapshot, ExampleHostMetricsTracker};
pub use watch::ReloadWatcher;

pub const WINDOW_TITLE: &str = "egui-component JSX Runtime";
pub const WINDOW_INNER_SIZE: [f32; 2] = [1360.0, 940.0];

pub fn install_context(ctx: &Context) {
    theme::install(
        ctx,
        ThemeSpec::preset(BaseColor::Neutral),
        ThemeMode::System,
    );
}

pub fn default_entry_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("runtime-jsx")
        .join("app.jsx")
}

pub fn entry_path_from_args() -> PathBuf {
    std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_entry_path)
}

pub fn run_from_args() -> eframe::Result {
    run_native(entry_path_from_args())
}

pub fn run_native(entry_path: PathBuf) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size(WINDOW_INNER_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        options,
        Box::new(move |creation_context| {
            install_context(&creation_context.egui_ctx);
            Ok(Box::new(RuntimeJsxApp::new(entry_path)))
        }),
    )
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        path::PathBuf,
        process::Command,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };

    use super::{default_entry_path, metrics::ExampleHostMetrics};
    use clay_jsx_egui_bridge::{
        JsxRuntimeDebugMetrics, JsxRuntimeSession, MotionFrame, MotionProperty,
    };
    use egui_component::contract::{
        ContractEvent, ContractLength, ContractNode, ContractOverflow, EventKind, EventValue,
        NodeId,
    };
    use tempfile::tempdir;

    const JSX_COMPONENT_PREVIEW_IDS: &[&str] = &[
        "jsx-preview-audio-playback",
        "jsx-preview-button",
        "jsx-preview-button-group",
        "jsx-preview-canva",
        "jsx-preview-card",
        "jsx-preview-checkbox",
        "jsx-preview-collab-cursor",
        "jsx-preview-collapsible",
        "jsx-preview-color",
        "jsx-preview-color-input",
        "jsx-preview-color-strip",
        "jsx-preview-combobox",
        "jsx-preview-command",
        "jsx-preview-context-menu",
        "jsx-preview-dialogue",
        "jsx-preview-drag-board",
        "jsx-preview-dropdown-menu",
        "jsx-preview-emoji-selector",
        "jsx-preview-field",
        "jsx-preview-file-tree",
        "jsx-preview-hierarchy",
        "jsx-preview-icon",
        "jsx-preview-icon-toolbar",
        "jsx-preview-image",
        "jsx-preview-image-tile",
        "jsx-preview-input",
        "jsx-preview-kbd",
        "jsx-preview-label",
        "jsx-preview-menu-bar",
        "jsx-preview-open-with",
        "jsx-preview-pagination",
        "jsx-preview-palette-color-input",
        "jsx-preview-palette-preview",
        "jsx-preview-popover",
        "jsx-preview-progress",
        "jsx-preview-radio",
        "jsx-preview-select",
        "jsx-preview-separator",
        "jsx-preview-sidebar",
        "jsx-preview-skeleton",
        "jsx-preview-slider",
        "jsx-preview-spinner",
        "jsx-preview-switch",
        "jsx-preview-tabs",
        "jsx-preview-toast",
        "jsx-preview-toggle-group",
        "jsx-preview-toolbar",
        "jsx-preview-tooltip",
        "jsx-preview-twemoji",
    ];

    #[test]
    fn default_jsx_file_renders_contract_tree() {
        let (_session, rendered) =
            JsxRuntimeSession::load(&default_entry_path()).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(tree.root.family_id().as_str(), "column");
    }

    #[test]
    fn default_jsx_file_demonstrates_effect_driven_runtime_updates() {
        let (mut session, rendered) =
            JsxRuntimeSession::load(&default_entry_path()).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert!(find_node(&tree.root, "runtime-effects-query").is_some());
        assert_eq!(
            label_text(
                find_node(&tree.root, "runtime-effects-deferred")
                    .expect("runtime effects deferred label should exist")
            ),
            Some("Deferred query: shader compiler")
        );

        let rendered = drain_async_until_update(&mut session, Duration::from_millis(100))
            .expect("mount effects should publish an initial external-store update");
        let tree = rendered.tree.expect("effect-driven update should rerender");
        assert_eq!(
            label_text(
                find_node(&tree.root, "runtime-effects-store-phase")
                    .expect("runtime effects store phase label should exist")
            ),
            Some("External store phase: live")
        );

        let rendered = drain_async_until(&mut session, Duration::from_millis(1200), |rendered| {
            let Some(tree) = rendered.tree.as_ref() else {
                return false;
            };
            label_text(
                find_node(&tree.root, "runtime-effects-store-tick")
                    .expect("runtime effects store tick label should exist"),
            ) == Some("Store tick: 1")
        })
        .expect("timer-driven update should rerender the example app");
        let tree = rendered
            .tree
            .expect("timer-driven update should return a tree");
        assert_eq!(
            label_text(
                find_node(&tree.root, "runtime-effects-async")
                    .expect("runtime effects async label should exist")
            ),
            Some("Async reducer: Resolved \"shader compiler\" after 15 token checks.")
        );
        assert_eq!(
            label_text(
                find_node(&tree.root, "runtime-effects-store-tick")
                    .expect("runtime effects store tick label should exist")
            ),
            Some("Store tick: 1")
        );
    }

    #[test]
    fn jsx_component_catalog_entrypoint_renders_components() {
        let catalog_path = default_entry_path().with_file_name("catalog.tsx");
        let (mut session, rendered) =
            JsxRuntimeSession::load(&catalog_path).expect("catalog tsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");

        assert_eq!(tree.root.family_id().as_str(), "column");
        assert_eq!(tree.root.node_id().as_str(), "jsx-migration-catalog-root");
        assert!(find_node(&tree.root, "jsx-component-authoring-preview").is_some());
        assert_eq!(
            find_node(&tree.root, "jsx-catalog-sidepanel-shell")
                .expect("catalog sidepanel shell")
                .family_id()
                .as_str(),
            "sized-box"
        );
        assert_eq!(
            find_node(&tree.root, "jsx-catalog-preview-shell")
                .expect("catalog preview shell")
                .family_id()
                .as_str(),
            "sized-box"
        );

        assert!(find_node(&tree.root, "jsx-catalog-nav-component-authoring").is_some());
        assert!(find_node(&tree.root, "jsx-catalog-nav-manifest-summary").is_some());
        for preview_id in JSX_COMPONENT_PREVIEW_IDS {
            let preview_name = preview_id
                .strip_prefix("jsx-preview-")
                .expect("preview ids should use the jsx-preview- prefix");
            let nav_id = format!("jsx-catalog-nav-{preview_name}");
            assert!(
                find_node(&tree.root, nav_id.as_str()).is_some(),
                "missing JSX component nav {nav_id}"
            );
        }

        let sidepanel = find_node(&tree.root, "jsx-catalog-sidepanel-shell")
            .expect("catalog sidepanel sized box");
        match sidepanel {
            ContractNode::SizedBox(props) => assert_eq!(props.width, Some(264.0)),
            _ => panic!("expected catalog sidepanel shell to be a sized box"),
        }
        let sidepanel_scroller = find_node(&tree.root, "jsx-catalog-sidepanel-scroller")
            .expect("catalog sidepanel scroller");
        assert_eq!(
            sidepanel_scroller
                .common()
                .layout
                .as_ref()
                .and_then(|layout| layout.height.as_ref())
                .cloned(),
            Some(ContractLength::Px { value: 640.0 })
        );
        assert_eq!(
            sidepanel_scroller
                .common()
                .layout
                .as_ref()
                .and_then(|layout| layout.overflow_y),
            Some(ContractOverflow::Scroll)
        );
        let preview_shell =
            find_node(&tree.root, "jsx-catalog-preview-shell").expect("catalog preview sized box");
        match preview_shell {
            ContractNode::SizedBox(props) => assert_eq!(props.width, Some(820.0)),
            _ => panic!("expected catalog preview shell to be a sized box"),
        }
        let nav_label = find_node(&tree.root, "jsx-catalog-nav-card").expect("catalog nav button");
        assert_eq!(
            nav_label
                .common()
                .layout
                .as_ref()
                .and_then(|layout| layout.width.as_ref())
                .cloned(),
            Some(ContractLength::Px { value: 232.0 })
        );
        assert_eq!(
            button_selected(
                find_node(&tree.root, "jsx-catalog-nav-component-authoring")
                    .expect("default nav button should exist")
            ),
            Some(true)
        );
        assert_eq!(button_selected(nav_label), Some(false));
        assert!(common_class(
            find_node(&tree.root, "jsx-catalog-preview-column")
                .expect("catalog preview column should exist")
        )
        .is_some_and(|class| class.contains("items-stretch")));
        assert_eq!(
            label_text(
                find_node(&tree.root, "jsx-catalog-active-label")
                    .expect("active preview label should exist")
            ),
            Some("Component authoring")
        );
        assert!(common_class(
            find_node(&tree.root, "jsx-component-authoring-preview")
                .expect("component authoring preview should exist")
        )
        .is_some_and(|class| class.contains("items-stretch")));
        let primary = find_node(&tree.root, "jsx-migration-primary-action")
            .expect("component authoring primary action");
        assert_eq!(button_variant(primary).as_deref(), Some("Primary"));
        assert!(
            common_class(primary).is_some_and(|class| class.contains("text-primary-foreground"))
        );
        assert!(find_node(&tree.root, "jsx-button-primary").is_none());

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-button",
                EventKind::Clicked,
            )])
            .expect("catalog nav click should rerender");
        let tree = rendered
            .tree
            .expect("catalog selection should return a tree");

        assert_eq!(
            label_text(
                find_node(&tree.root, "jsx-catalog-active-label")
                    .expect("active preview label should update")
            ),
            Some("Button")
        );
        assert_eq!(
            button_selected(
                find_node(&tree.root, "jsx-catalog-nav-component-authoring")
                    .expect("component authoring nav button should still exist")
            ),
            Some(false)
        );
        assert_eq!(
            button_selected(
                find_node(&tree.root, "jsx-catalog-nav-button")
                    .expect("button nav button should exist")
            ),
            Some(true)
        );
        assert!(find_node(&tree.root, "jsx-component-authoring-preview").is_none());
        assert!(find_node(&tree.root, "jsx-card").is_none());

        let primary = find_node(&tree.root, "jsx-button-primary").expect("preview primary button");
        assert_eq!(button_variant(primary).as_deref(), Some("Primary"));
        assert!(
            common_class(primary).is_some_and(|class| class.contains("text-primary-foreground"))
        );

        let selected = find_node(&tree.root, "jsx-button-selected").expect("selected button");
        assert_eq!(button_selected(selected), Some(true));
        assert!(common_class(selected).is_some_and(|class| class.contains("font-semibold")));

        let disabled = find_node(&tree.root, "jsx-button-disabled").expect("disabled button");
        assert_eq!(disabled.common().enabled, false);
        assert!(common_class(disabled).is_some_and(|class| class.contains("text-muted-foreground")));

        assert_eq!(checkbox_value(&tree.root, "jsx-checkbox"), None);
    }

    #[test]
    fn jsx_component_catalog_checkbox_preview_round_trips_toggle_state() {
        let catalog_path = default_entry_path().with_file_name("catalog.tsx");
        let (mut session, rendered) =
            JsxRuntimeSession::load(&catalog_path).expect("catalog tsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");

        assert!(find_node(&tree.root, "jsx-checkbox").is_none());

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-checkbox",
                EventKind::Clicked,
            )])
            .expect("catalog nav click should rerender");
        let tree = rendered
            .tree
            .expect("checkbox selection should return a tree");

        assert_eq!(checkbox_value(&tree.root, "jsx-checkbox"), Some(true));

        let rendered = session
            .dispatch_events(&[ContractEvent::new("jsx-checkbox", EventKind::Toggled)
                .value(Some(EventValue::Boolean(false)))])
            .expect("checkbox toggle should rerender");
        let tree = rendered
            .tree
            .expect("checkbox toggle should produce a new tree");

        assert_eq!(checkbox_value(&tree.root, "jsx-checkbox"), Some(false));
    }

    #[test]
    fn jsx_component_catalog_input_style_previews_round_trip_changed_values() {
        let catalog_path = default_entry_path().with_file_name("catalog.tsx");
        let (mut session, _) =
            JsxRuntimeSession::load(&catalog_path).expect("catalog tsx file should render");

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-canva",
                EventKind::Clicked,
            )])
            .expect("canva nav click should rerender");
        let tree = rendered.tree.expect("canva preview should return a tree");
        assert_eq!(
            number_input_value(&tree.root, "jsx-canva-axis-x-X"),
            Some(120.0)
        );
        assert_eq!(
            input_value(&tree.root, "jsx-canva-color-stop-input"),
            Some("#2896ff")
        );

        let rendered = session
            .dispatch_events(
                &[ContractEvent::new("jsx-canva-axis-x-X", EventKind::Changed)
                    .value(Some(EventValue::Number(144.0)))],
            )
            .expect("canva axis change should rerender");
        let tree = rendered
            .tree
            .expect("canva axis change should return a tree");
        assert_eq!(
            number_input_value(&tree.root, "jsx-canva-axis-x-X"),
            Some(144.0)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-canva-color-stop-input",
                EventKind::Changed,
            )
            .value(Some(EventValue::Text("#f59e0b".into())))])
            .expect("canva color change should rerender");
        let tree = rendered
            .tree
            .expect("canva color change should return a tree");
        assert_eq!(
            input_value(&tree.root, "jsx-canva-color-stop-input"),
            Some("#f59e0b")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-input",
                EventKind::Clicked,
            )])
            .expect("input nav click should rerender");
        let tree = rendered.tree.expect("input preview should return a tree");
        assert_eq!(
            input_value(&tree.root, "jsx-input"),
            Some("className support")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("jsx-input", EventKind::Changed)
                .value(Some(EventValue::Text("runtime state ok".into())))])
            .expect("input change should rerender");
        let tree = rendered.tree.expect("input change should return a tree");
        assert_eq!(
            input_value(&tree.root, "jsx-input"),
            Some("runtime state ok")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-field",
                EventKind::Clicked,
            )])
            .expect("field nav click should rerender");
        let tree = rendered.tree.expect("field preview should return a tree");
        assert_eq!(
            input_value(&tree.root, "jsx-field-control"),
            Some("Runtime JSX")
        );

        let rendered = session
            .dispatch_events(
                &[ContractEvent::new("jsx-field-control", EventKind::Changed)
                    .value(Some(EventValue::Text("Preview shell".into())))],
            )
            .expect("field change should rerender");
        let tree = rendered.tree.expect("field change should return a tree");
        assert_eq!(
            input_value(&tree.root, "jsx-field-control"),
            Some("Preview shell")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-color-input",
                EventKind::Clicked,
            )])
            .expect("color input nav click should rerender");
        let tree = rendered
            .tree
            .expect("color input preview should return a tree");
        assert_eq!(
            input_value(&tree.root, "jsx-color-input-value"),
            Some("#10b981")
        );

        let rendered = session
            .dispatch_events(&[
                ContractEvent::new("jsx-color-input-value", EventKind::Changed)
                    .value(Some(EventValue::Text("#0f172a".into()))),
            ])
            .expect("color input change should rerender");
        let tree = rendered
            .tree
            .expect("color input change should return a tree");
        assert_eq!(
            input_value(&tree.root, "jsx-color-input-value"),
            Some("#0f172a")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-palette-color-input",
                EventKind::Clicked,
            )])
            .expect("palette color input nav click should rerender");
        let tree = rendered
            .tree
            .expect("palette color input preview should return a tree");
        assert_eq!(
            input_value(&tree.root, "jsx-palette-color-input-custom-input-value"),
            Some("#f43f5e")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-palette-color-input-custom-input-value",
                EventKind::Changed,
            )
            .value(Some(EventValue::Text("#22c55e".into())))])
            .expect("palette color input change should rerender");
        let tree = rendered
            .tree
            .expect("palette color input change should return a tree");
        assert_eq!(
            input_value(&tree.root, "jsx-palette-color-input-custom-input-value"),
            Some("#22c55e")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-slider",
                EventKind::Clicked,
            )])
            .expect("slider nav click should rerender");
        let tree = rendered.tree.expect("slider preview should return a tree");
        assert_eq!(slider_value(&tree.root, "jsx-slider"), Some(42.0));
        assert_eq!(
            number_input_value(&tree.root, "jsx-number-input"),
            Some(42.0)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("jsx-slider", EventKind::Changed)
                .value(Some(EventValue::Number(64.0)))])
            .expect("slider change should rerender");
        let tree = rendered.tree.expect("slider change should return a tree");
        assert_eq!(slider_value(&tree.root, "jsx-slider"), Some(64.0));
        assert_eq!(
            number_input_value(&tree.root, "jsx-number-input"),
            Some(64.0)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("jsx-number-input", EventKind::Changed)
                .value(Some(EventValue::Number(72.0)))])
            .expect("number input change should rerender");
        let tree = rendered
            .tree
            .expect("number input change should return a tree");
        assert_eq!(slider_value(&tree.root, "jsx-slider"), Some(72.0));
        assert_eq!(
            number_input_value(&tree.root, "jsx-number-input"),
            Some(72.0)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-switch",
                EventKind::Clicked,
            )])
            .expect("switch nav click should rerender");
        let tree = rendered.tree.expect("switch preview should return a tree");
        assert_eq!(switch_value(&tree.root, "jsx-switch"), Some(true));

        let rendered = session
            .dispatch_events(&[ContractEvent::new("jsx-switch", EventKind::Toggled)
                .value(Some(EventValue::Boolean(false)))])
            .expect("switch toggle should rerender");
        let tree = rendered.tree.expect("switch toggle should return a tree");
        assert_eq!(switch_value(&tree.root, "jsx-switch"), Some(false));
    }

    #[test]
    fn jsx_component_catalog_selection_previews_round_trip_selected_state() {
        let catalog_path = default_entry_path().with_file_name("catalog.tsx");
        let (mut session, _) =
            JsxRuntimeSession::load(&catalog_path).expect("catalog tsx file should render");

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-button-group",
                EventKind::Clicked,
            )])
            .expect("button group nav click should rerender");
        let tree = rendered
            .tree
            .expect("button group preview should return a tree");
        assert_eq!(
            button_selected(
                find_node(&tree.root, "jsx-button-group-team").expect("default button group item")
            ),
            Some(true)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-button-group-enterprise",
                EventKind::Clicked,
            )])
            .expect("button group click should rerender");
        let tree = rendered
            .tree
            .expect("button group click should return a tree");
        assert_eq!(
            button_selected(
                find_node(&tree.root, "jsx-button-group-enterprise")
                    .expect("enterprise button group item")
            ),
            Some(true)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-select",
                EventKind::Clicked,
            )])
            .expect("select nav click should rerender");
        let tree = rendered.tree.expect("select preview should return a tree");
        assert_eq!(
            button_label(
                find_node(&tree.root, "jsx-select-trigger").expect("select trigger should exist")
            ),
            Some("Team")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-select-option-enterprise",
                EventKind::Clicked,
            )])
            .expect("select option click should rerender");
        let tree = rendered.tree.expect("select click should return a tree");
        assert_eq!(
            button_label(
                find_node(&tree.root, "jsx-select-trigger").expect("select trigger should exist")
            ),
            Some("Enterprise")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-combobox",
                EventKind::Clicked,
            )])
            .expect("combobox nav click should rerender");
        let tree = rendered
            .tree
            .expect("combobox preview should return a tree");
        assert_eq!(
            button_label(
                find_node(&tree.root, "jsx-combobox-trigger")
                    .expect("combobox trigger should exist")
            ),
            Some("Team")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-combobox-option-enterprise",
                EventKind::Clicked,
            )])
            .expect("combobox option click should rerender");
        let tree = rendered.tree.expect("combobox click should return a tree");
        assert_eq!(
            button_label(
                find_node(&tree.root, "jsx-combobox-trigger")
                    .expect("combobox trigger should exist")
            ),
            Some("Team, Enterprise")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-radio",
                EventKind::Clicked,
            )])
            .expect("radio nav click should rerender");
        let tree = rendered.tree.expect("radio preview should return a tree");
        assert_eq!(radio_value(&tree.root, "jsx-radio"), Some(false));
        assert_eq!(radio_value(&tree.root, "jsx-radio-group-team"), Some(true));

        let rendered = session
            .dispatch_events(&[ContractEvent::new("jsx-radio", EventKind::Toggled)
                .value(Some(EventValue::Boolean(true)))])
            .expect("standalone radio toggle should rerender");
        let tree = rendered
            .tree
            .expect("standalone radio toggle should return a tree");
        assert_eq!(radio_value(&tree.root, "jsx-radio"), Some(true));

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-radio-group-enterprise",
                EventKind::Toggled,
            )
            .value(Some(EventValue::Boolean(true)))])
            .expect("radio group toggle should rerender");
        let tree = rendered
            .tree
            .expect("radio group toggle should return a tree");
        assert_eq!(
            radio_value(&tree.root, "jsx-radio-group-enterprise"),
            Some(true)
        );
        assert_eq!(radio_value(&tree.root, "jsx-radio-group-team"), Some(false));

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-tabs",
                EventKind::Clicked,
            )])
            .expect("tabs nav click should rerender");
        let tree = rendered.tree.expect("tabs preview should return a tree");
        assert_eq!(
            button_selected(find_node(&tree.root, "jsx-tabs-activity").expect("default tab")),
            Some(true)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("jsx-tabs-settings", EventKind::Clicked)])
            .expect("tab click should rerender");
        let tree = rendered.tree.expect("tab click should return a tree");
        assert_eq!(
            button_selected(find_node(&tree.root, "jsx-tabs-settings").expect("settings tab")),
            Some(true)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-toggle-group",
                EventKind::Clicked,
            )])
            .expect("toggle group nav click should rerender");
        let tree = rendered
            .tree
            .expect("toggle group preview should return a tree");
        assert_eq!(
            button_selected(
                find_node(&tree.root, "jsx-toggle-group-team").expect("default toggle group item")
            ),
            Some(true)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-toggle-group-enterprise",
                EventKind::Clicked,
            )])
            .expect("toggle group click should rerender");
        let tree = rendered
            .tree
            .expect("toggle group click should return a tree");
        assert_eq!(
            button_selected(
                find_node(&tree.root, "jsx-toggle-group-enterprise")
                    .expect("enterprise toggle group item")
            ),
            Some(true)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-icon-toolbar",
                EventKind::Clicked,
            )])
            .expect("icon toolbar nav click should rerender");
        let tree = rendered
            .tree
            .expect("icon toolbar preview should return a tree");
        assert_eq!(
            button_selected(
                find_node(&tree.root, "jsx-icon-toolbar-move").expect("default icon toolbar item")
            ),
            Some(true)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-icon-toolbar-rotate",
                EventKind::Clicked,
            )])
            .expect("icon toolbar click should rerender");
        let tree = rendered
            .tree
            .expect("icon toolbar click should return a tree");
        assert_eq!(
            button_selected(
                find_node(&tree.root, "jsx-icon-toolbar-rotate").expect("rotate icon toolbar item")
            ),
            Some(true)
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-file-tree",
                EventKind::Clicked,
            )])
            .expect("file tree nav click should rerender");
        let tree = rendered
            .tree
            .expect("file tree preview should return a tree");
        assert_eq!(
            label_text(
                find_node(&tree.root, "jsx-file-tree-selection")
                    .expect("file tree selection label should exist")
            ),
            Some("Selected item: button")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-file-tree-workspace",
                EventKind::Clicked,
            )])
            .expect("file tree click should rerender");
        let tree = rendered.tree.expect("file tree click should return a tree");
        assert_eq!(
            label_text(
                find_node(&tree.root, "jsx-file-tree-selection")
                    .expect("file tree selection label should exist")
            ),
            Some("Selected item: workspace")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-hierarchy",
                EventKind::Clicked,
            )])
            .expect("hierarchy nav click should rerender");
        let tree = rendered
            .tree
            .expect("hierarchy preview should return a tree");
        assert_eq!(
            label_text(
                find_node(&tree.root, "jsx-hierarchy-selection")
                    .expect("hierarchy selection label should exist")
            ),
            Some("Selected item: player")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-hierarchy-scene",
                EventKind::Clicked,
            )])
            .expect("hierarchy click should rerender");
        let tree = rendered.tree.expect("hierarchy click should return a tree");
        assert_eq!(
            label_text(
                find_node(&tree.root, "jsx-hierarchy-selection")
                    .expect("hierarchy selection label should exist")
            ),
            Some("Selected item: scene")
        );
    }

    #[test]
    fn jsx_component_catalog_menu_previews_round_trip_command_state() {
        let catalog_path = default_entry_path().with_file_name("catalog.tsx");
        let (mut session, _) =
            JsxRuntimeSession::load(&catalog_path).expect("catalog tsx file should render");

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-command",
                EventKind::Clicked,
            )])
            .expect("command nav click should rerender");
        let tree = rendered.tree.expect("command preview should return a tree");
        assert_eq!(
            label_text(
                find_node(&tree.root, "jsx-command-selection")
                    .expect("command selection label should exist")
            ),
            Some("Last command: Open command menu")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("command-item-theme", EventKind::Clicked)])
            .expect("command item click should rerender");
        let tree = rendered
            .tree
            .expect("command item click should return a tree");
        assert_eq!(
            label_text(
                find_node(&tree.root, "jsx-command-selection")
                    .expect("command selection label should exist")
            ),
            Some("Last command: theme")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-dropdown-menu",
                EventKind::Clicked,
            )])
            .expect("dropdown nav click should rerender");
        let tree = rendered
            .tree
            .expect("dropdown preview should return a tree");
        assert_eq!(
            label_text(
                find_node(&tree.root, "jsx-dropdown-menu-selection")
                    .expect("dropdown selection label should exist")
            ),
            Some("Last action: profile")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-dropdown-menu-trigger",
                EventKind::Clicked,
            )])
            .expect("dropdown trigger click should rerender");
        let tree = rendered
            .tree
            .expect("dropdown trigger click should return a tree");
        assert!(find_node(&tree.root, "jsx-dropdown-menu-items-settings-button").is_some());

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-dropdown-menu-items-settings-button",
                EventKind::Clicked,
            )])
            .expect("dropdown settings click should rerender");
        let tree = rendered
            .tree
            .expect("dropdown settings click should return a tree");
        assert_eq!(
            label_text(
                find_node(&tree.root, "jsx-dropdown-menu-selection")
                    .expect("dropdown selection label should exist")
            ),
            Some("Last action: settings")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-catalog-nav-open-with",
                EventKind::Clicked,
            )])
            .expect("open with nav click should rerender");
        let tree = rendered
            .tree
            .expect("open with preview should return a tree");
        assert_eq!(
            button_label(
                find_node(&tree.root, "jsx-open-with-action")
                    .expect("open with action button should exist")
            ),
            Some("Profile")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("jsx-open-with-menu", EventKind::Clicked)])
            .expect("open with menu click should rerender");
        let tree = rendered
            .tree
            .expect("open with menu click should return a tree");
        assert!(find_node(&tree.root, "jsx-open-with-items-settings-button").is_some());

        let rendered = session
            .dispatch_events(&[ContractEvent::new(
                "jsx-open-with-items-settings-button",
                EventKind::Clicked,
            )])
            .expect("open with settings click should rerender");
        let tree = rendered
            .tree
            .expect("open with settings click should return a tree");
        assert_eq!(
            button_label(
                find_node(&tree.root, "jsx-open-with-action")
                    .expect("open with action button should exist")
            ),
            Some("Settings")
        );
    }

    #[test]
    fn dispatches_egui_events_back_into_jsx_hooks() {
        let (mut session, rendered) =
            JsxRuntimeSession::load(&default_entry_path()).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "snap-checkbox"), Some(true));

        let event = ContractEvent::new("snap-checkbox", EventKind::Toggled)
            .value(Some(EventValue::Boolean(false)));
        let rendered = session
            .dispatch_events(&[event])
            .expect("event dispatch should rerender");
        let tree = rendered.tree.expect("changed render should return a tree");

        assert_eq!(checkbox_value(&tree.root, "snap-checkbox"), Some(false));
    }

    #[test]
    fn no_op_rerender_skips_contract_tree_materialization() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("noop.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [checked, setChecked] = useState(false);
  return (
    <div id="root" data-slot="column">
      <input id="toggle" type="checkbox" checked={checked} onToggle={(event, value) => setChecked(value)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(checkbox_value(&tree.root, "toggle"), Some(false));

        let event = ContractEvent::new("toggle", EventKind::Toggled)
            .value(Some(EventValue::Boolean(false)));
        let rendered = session
            .dispatch_events(&[event])
            .expect("event dispatch should return current tree");

        assert!(rendered.tree.is_none());
    }

    #[test]
    fn dynamic_jsx_lists_apply_incremental_insert_remove_and_reorder() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("dynamic-list.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [items, setItems] = useState(["a", "c"]);
  return (
    <div id="root" data-slot="column">
      <button id="change" label="Change" onClick={() => setItems(["c", "b"])} />
      {items.map((item) => <label key={item} id={item} text={item} />)}
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            contract_children(&tree.root)
                .iter()
                .map(|child| child.node_id().as_str())
                .collect::<Vec<_>>(),
            vec!["change", "a", "c"]
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("change", EventKind::Clicked)])
            .expect("event dispatch should rerender dynamic list");
        let tree = rendered.tree.expect("changed render should return a tree");

        assert_eq!(
            contract_children(&tree.root)
                .iter()
                .map(|child| child.node_id().as_str())
                .collect::<Vec<_>>(),
            vec!["change", "c", "b"]
        );
    }

    #[test]
    fn keyed_reorder_keeps_handlers_attached_to_the_same_node_ids() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("reorder-handlers.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [items, setItems] = useState(["alpha", "beta"]);
  const [selected, setSelected] = useState("none");
  return (
    <div id="root" data-slot="column">
      <button id="swap" label="Swap" onClick={() => setItems(["beta", "alpha"])} />
      {items.map((item) => (
        <button
          key={item}
          id={item}
          label={item}
          onClick={() => setSelected(item)}
        />
      ))}
      <label id="selected" text={selected} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, _) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");

        let rendered = session
            .dispatch_events(&[ContractEvent::new("swap", EventKind::Clicked)])
            .expect("swap event should rerender");
        let tree = rendered
            .tree
            .expect("reordered render should return a tree");
        assert_eq!(
            contract_children(&tree.root)
                .iter()
                .map(|child| child.node_id().as_str())
                .collect::<Vec<_>>(),
            vec!["swap", "beta", "alpha", "selected"]
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("alpha", EventKind::Clicked)])
            .expect("reordered alpha click should rerender");
        let tree = rendered
            .tree
            .expect("selection render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "selected").expect("selected label should exist")),
            Some("alpha")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("beta", EventKind::Clicked)])
            .expect("reordered beta click should rerender");
        let tree = rendered
            .tree
            .expect("selection render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "selected").expect("selected label should exist")),
            Some("beta")
        );
    }

    #[test]
    fn stable_jsx_node_ids_can_move_between_parents() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("move-between-parents.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [right, setRight] = useState(false);
  return (
    <div id="root" data-slot="row">
      <div id="left" data-slot="column">{!right && <label id="moving" text="Moving" />}</div>
      <div id="right" data-slot="column">{right && <label id="moving" text="Moving" />}</div>
      <button id="move" label="Move" onClick={() => setRight(true)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            contract_children(find_node(&tree.root, "left").expect("left should exist"))
                .iter()
                .map(|child| child.node_id().as_str())
                .collect::<Vec<_>>(),
            vec!["moving"]
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("move", EventKind::Clicked)])
            .expect("event dispatch should move child");
        let tree = rendered.tree.expect("changed render should return a tree");

        assert!(
            contract_children(find_node(&tree.root, "left").expect("left should exist")).is_empty()
        );
        assert_eq!(
            contract_children(find_node(&tree.root, "right").expect("right should exist"))
                .iter()
                .map(|child| child.node_id().as_str())
                .collect::<Vec<_>>(),
            vec!["moving"]
        );
    }

    #[test]
    fn prop_updates_change_one_leaf_without_remounting_siblings() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("prop-update.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [text, setText] = useState("old");
  return (
    <div id="root" data-slot="column">
      <button id="change" label="Change" onClick={() => setText("new")} />
      <label id="target" text={text} />
      <label id="sibling" text="stable" />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert_eq!(
            label_text(find_node(&tree.root, "target").unwrap()),
            Some("old")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "sibling").unwrap()),
            Some("stable")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("change", EventKind::Clicked)])
            .expect("event dispatch should update label");
        let tree = rendered.tree.expect("changed render should return a tree");

        assert_eq!(
            label_text(find_node(&tree.root, "target").unwrap()),
            Some("new")
        );
        assert_eq!(
            label_text(find_node(&tree.root, "sibling").unwrap()),
            Some("stable")
        );
    }

    #[test]
    fn removed_node_handlers_are_not_dispatched() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("handler-cleanup.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";

function App() {
  const [visible, setVisible] = useState(true);
  const [count, setCount] = useState(0);
  return (
    <div id="root" data-slot="column">
      <button id="remove" label="Remove" onClick={() => setVisible(false)} />
      {visible && <button id="target" label="Target" onClick={() => setCount(count + 1)} />}
      <label id="count" text={String(count)} />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        let tree = rendered.tree.expect("initial render should return a tree");
        assert!(find_node(&tree.root, "target").is_some());
        assert_eq!(
            label_text(find_node(&tree.root, "count").unwrap()),
            Some("0")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("remove", EventKind::Clicked)])
            .expect("remove event should rerender");
        let tree = rendered.tree.expect("changed render should return a tree");
        assert!(find_node(&tree.root, "target").is_none());
        assert_eq!(
            label_text(find_node(&tree.root, "count").unwrap()),
            Some("0")
        );

        let rendered = session
            .dispatch_events(&[ContractEvent::new("target", EventKind::Clicked)])
            .expect("stale target event should be ignored");
        assert!(rendered.tree.is_none());
    }

    #[test]
    fn motion_react_import_commits_retained_motion_values() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render } from "egui";
import { motion } from "motion/react";

function App() {
  return (
    <motion.div
      data-slot="card"
      id="panel"
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 1, ease: "linear" }}
    >
      <label id="copy" text="Animated" />
    </motion.div>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        assert_eq!(
            rendered
                .tree
                .expect("initial render should return a tree")
                .root
                .family_id()
                .as_str(),
            "card"
        );
        assert!(rendered.motion.active);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::Opacity, 0.0);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::X, -10.0);

        let rendered = session
            .tick_motion(0.0)
            .expect("motion tick should succeed");
        assert!(rendered.tree.is_none());
        assert_motion_close(&rendered.motion, "panel", MotionProperty::Opacity, 0.0);

        let rendered = session
            .tick_motion(0.5)
            .expect("motion tick should advance");
        assert!(rendered.tree.is_none());
        assert!(rendered.motion.active);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::Opacity, 0.5);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::X, -5.0);

        let rendered = session.tick_motion(1.0).expect("motion tick should finish");
        assert!(!rendered.motion.active);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::Opacity, 1.0);
        assert_motion_close(&rendered.motion, "panel", MotionProperty::X, 0.0);
    }

    #[test]
    fn react_motion_alias_updates_motion_without_contract_tree_materialization() {
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("motion-update.jsx");
        std::fs::write(
            &entry_path,
            r#"
import { render, useState } from "egui";
import { motion } from "react/motion";

function App() {
  const [open, setOpen] = useState(false);
  return (
    <div id="root" data-slot="column">
      <button id="toggle" label="Toggle" onClick={() => setOpen(true)} />
      <motion.label
        id="status"
        text="Status"
        animate={{ opacity: open ? 1 : 0 }}
        transition={{ duration: 1, ease: "linear" }}
      />
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("jsx file should be written");

        let (mut session, rendered) =
            JsxRuntimeSession::load(&entry_path).expect("jsx file should render");
        assert!(!rendered.motion.active);
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 0.0);

        let rendered = session
            .dispatch_events(&[ContractEvent::new("toggle", EventKind::Clicked)])
            .expect("event dispatch should retarget motion");
        assert!(rendered.tree.is_none());
        assert!(rendered.motion.active);
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 0.0);

        let rendered = session.tick_motion(0.0).expect("motion tick should start");
        assert!(rendered.tree.is_none());
        let rendered = session
            .tick_motion(0.25)
            .expect("motion tick should advance");
        assert!(rendered.tree.is_none());
        assert_motion_close(&rendered.motion, "status", MotionProperty::Opacity, 0.25);
    }

    #[test]
    #[ignore = "ship-readiness stress harness"]
    fn jsx_runtime_stress_harness() {
        let event_batches = read_env_u64("CLAY_JSX_STRESS_EVENT_BATCHES", 1_000) as usize;
        let reload_cycles = read_env_u64("CLAY_JSX_STRESS_RELOAD_CYCLES", 100) as usize;
        let mount_cycles = read_env_u64("CLAY_JSX_STRESS_MOUNT_CYCLES", 100) as usize;
        let command = "cargo test --example runtime-jsx-host jsx_runtime_stress_harness -- --ignored --nocapture";
        let dir = tempdir().expect("temp dir should be created");
        let entry_path = dir.path().join("stress.tsx");
        let copy_path = dir.path().join("copy.tsx");
        write_stress_copy(&copy_path, "v0");
        std::fs::write(
            &entry_path,
            r#"
import { render, useEffect, useState, useSyncExternalStore } from "egui";
import { stressCopy } from "./copy.tsx";

const items = Array.from({ length: 240 }, (_, index) => `stress-item-${String(index)}`);

function createStore() {
  let snapshot = 0;
  const listeners = new Set();
  return {
    getSnapshot() {
      return snapshot;
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    emit(nextValue) {
      snapshot = nextValue;
      for (const listener of listeners) {
        listener();
      }
    },
  };
}

const store = createStore();

function App() {
  const [reversed, setReversed] = useState(false);
  const [checked, setChecked] = useState(false);
  const [visible, setVisible] = useState(true);
  const tick = useSyncExternalStore(store.subscribe, store.getSnapshot, store.getSnapshot);

  useEffect(() => {
    const handle = setInterval(() => {
      store.emit(store.getSnapshot() + 1);
    }, 20);
    return () => clearInterval(handle);
  }, []);

  const ordered = reversed ? [...items].reverse() : items;

  return (
    <div id="root" data-slot="column">
      <button id="toggle-order" label="Toggle order" onClick={() => setReversed((value) => !value)} />
      <button id="toggle-visible" label="Toggle visible" onClick={() => setVisible((value) => !value)} />
      <button id="noop" label="No-op" onClick={() => setChecked((value) => value)} />
      <input
        id="checked"
        type="checkbox"
        checked={checked}
        onToggle={(event, value) => setChecked(Boolean(value))}
      />
      <label id="stress-copy" text={stressCopy} />
      <label id="stress-tick" text={String(tick)} />
      {visible && (
        <div id="stress-list" data-slot="column">
          {ordered.map((item) => (
            <label key={item} id={item} text={item} />
          ))}
        </div>
      )}
    </div>
  );
}

render(<App />);
"#,
        )
        .expect("stress entry should be written");

        let (initial_session, _rendered) =
            JsxRuntimeSession::load(&entry_path).expect("stress session should load");
        let mut session = Some(initial_session);
        let mut totals = StressHarnessTotals::default();
        let mut reload_teardown_metrics = Vec::new();
        let mut all_teardown_metrics = Vec::new();

        for index in 0..event_batches {
            let rendered = session
                .as_mut()
                .expect("stress session should stay loaded")
                .dispatch_events(&[ContractEvent::new("toggle-order", EventKind::Clicked)])
                .expect("reorder dispatch should succeed");
            if let Some(tree) = rendered.tree {
                assert!(find_node(&tree.root, "stress-list").is_some());
            }

            let rendered = session
                .as_mut()
                .expect("stress session should stay loaded")
                .dispatch_events(&[ContractEvent::new("checked", EventKind::Toggled)
                    .value(Some(EventValue::Boolean(index % 2 == 0)))])
                .expect("checkbox dispatch should succeed");
            if let Some(tree) = rendered.tree {
                assert!(find_node(&tree.root, "checked").is_some());
            }

            let rendered = session
                .as_mut()
                .expect("stress session should stay loaded")
                .dispatch_events(&[ContractEvent::new("noop", EventKind::Clicked)])
                .expect("noop dispatch should succeed");
            assert!(rendered.tree.is_none());

            if index % 25 == 0 {
                let _ = session
                    .as_mut()
                    .expect("stress session should stay loaded")
                    .dispatch_events(&[ContractEvent::new("toggle-visible", EventKind::Clicked)])
                    .expect("visibility toggle should succeed");
            }

            let _ = drain_async_until_update(
                session.as_mut().expect("stress session should stay loaded"),
                Duration::from_millis(30),
            );
        }

        for reload_index in 0..reload_cycles {
            write_stress_copy(&copy_path, format!("v{}", reload_index + 1).as_str());
            let mut current = session
                .take()
                .expect("stress reload should have a live session");
            let _ = current.teardown().expect("stress reload should tear down");
            let metrics = current.debug_metrics();
            totals.record(&metrics);
            reload_teardown_metrics.push(metrics.clone());
            all_teardown_metrics.push(metrics);
            drop(current);
            let (next_session, rendered) =
                JsxRuntimeSession::load(&entry_path).expect("stress reload should recover");
            let tree = rendered.tree.expect("stress reload should render");
            let expected_copy = format!("v{}", reload_index + 1);
            assert_eq!(
                label_text(
                    find_node(&tree.root, "stress-copy")
                        .expect("stress copy label should exist after reload")
                ),
                Some(expected_copy.as_str())
            );
            session = Some(next_session);
        }

        for _ in 0..mount_cycles {
            let (mut mounted, _rendered) =
                JsxRuntimeSession::load(&entry_path).expect("mount/unmount cycle should load");
            let _ = mounted
                .teardown()
                .expect("mount/unmount cycle should tear down");
            let metrics = mounted.debug_metrics();
            all_teardown_metrics.push(metrics);
        }

        let mut session = session.expect("stress harness should retain a final session");
        let _ = session
            .teardown()
            .expect("final stress teardown should succeed");
        let final_metrics = session.debug_metrics();
        totals.record(&final_metrics);
        all_teardown_metrics.push(final_metrics.clone());

        let total_updates = totals.contract_tree_materialization_count
            + totals.contract_tree_noop_update_count
            + totals.motion_only_update_count;
        let mut gates = BTreeMap::new();
        gates.insert(
            "no_active_timers_after_teardown".to_owned(),
            all_teardown_metrics
                .iter()
                .all(|metrics| metrics.runtime.active_timer_count == 0),
        );
        gates.insert(
            "no_pending_wake_after_teardown".to_owned(),
            all_teardown_metrics
                .iter()
                .all(|metrics| !metrics.runtime.pending_host_wake),
        );
        gates.insert(
            "materialization_short_circuiting_observed".to_owned(),
            totals.contract_tree_materialization_count < total_updates,
        );
        gates.insert(
            "reloads_leave_no_dead_session_timers".to_owned(),
            reload_teardown_metrics.iter().all(|metrics| {
                metrics.runtime.active_timer_count == 0 && !metrics.runtime.pending_host_wake
            }),
        );
        let artifact = ReadinessArtifact {
            git_sha: git_sha(),
            timestamp_unix_secs: unix_timestamp_secs(),
            scenario: "stress".to_owned(),
            command: command.to_owned(),
            passed: gates.values().all(|passed| *passed),
            gates,
            metrics: ReadinessArtifactMetrics {
                runtime: serde_json::to_value(&final_metrics.runtime)
                    .expect("runtime metrics should serialize"),
                bridge: final_metrics.clone(),
                host: ExampleHostMetrics::default(),
                extras: serde_json::json!({
                    "event_batches": event_batches,
                    "reload_cycles": reload_cycles,
                    "mount_cycles": mount_cycles,
                    "totals": totals,
                    "reload_teardown_count": reload_teardown_metrics.len(),
                    "all_teardown_count": all_teardown_metrics.len(),
                    "total_update_count": total_updates,
                }),
            },
            samples: Vec::new(),
        };
        write_readiness_artifact("stress.json", &artifact, None);
        assert!(
            artifact.passed,
            "stress readiness gates failed: {}",
            serde_json::to_string_pretty(&artifact)
                .unwrap_or_else(|error| format!("failed to serialize stress artifact: {error}"))
        );
    }

    #[test]
    #[ignore = "long-running ship-readiness soak harness"]
    fn jsx_runtime_soak_harness() {
        let sample_interval = Duration::from_secs(read_env_u64("CLAY_JSX_SOAK_SAMPLE_SECS", 5));
        let idle_duration = Duration::from_secs(read_env_u64("CLAY_JSX_SOAK_IDLE_SECS", 15 * 60));
        let active_duration =
            Duration::from_secs(read_env_u64("CLAY_JSX_SOAK_ACTIVE_SECS", 15 * 60));
        let reload_duration =
            Duration::from_secs(read_env_u64("CLAY_JSX_SOAK_RELOAD_SECS", 5 * 60));
        let rss_band_kb = read_env_u64("CLAY_JSX_SOAK_RSS_BAND_KB", 64 * 1024);
        let idle_cpu_threshold = read_env_f64("CLAY_JSX_SOAK_IDLE_CPU_THRESHOLD", 5.0);
        let command = "cargo test --example runtime-jsx-host jsx_runtime_soak_harness -- --ignored --nocapture";
        let mut all_teardown_metrics = Vec::new();
        let mut reload_teardown_metrics = Vec::new();

        let motion_path = default_entry_path().with_file_name("motion-sync.tsx");
        let (mut motion_session, _rendered) =
            JsxRuntimeSession::load(&motion_path).expect("motion session should load");
        let rendered = motion_session
            .dispatch_events(&[ContractEvent::new("motion-toggle", EventKind::Clicked)])
            .expect("motion should retarget");
        assert!(rendered.motion.active);
        let mut now_secs = 0.0;
        let mut motion_frame = rendered.motion;
        while motion_frame.active {
            now_secs += 1.0 / 30.0;
            motion_frame = motion_session
                .tick_motion(now_secs)
                .expect("motion tick should succeed")
                .motion;
        }

        let mut idle_samples = Vec::new();
        sample_phase(
            "motion-idle",
            idle_duration,
            sample_interval,
            &mut motion_session,
            |session| {
                assert!(session
                    .drain_pending_runtime_updates()
                    .expect("idle drain should succeed")
                    .is_none());
            },
            &mut idle_samples,
        );
        let _ = motion_session
            .teardown()
            .expect("motion soak teardown should succeed");
        let motion_metrics = motion_session.debug_metrics();
        all_teardown_metrics.push(motion_metrics.clone());

        let app_path = default_entry_path();
        let (mut app_session, _rendered) =
            JsxRuntimeSession::load(&app_path).expect("default app should load");
        let mut active_samples = Vec::new();
        sample_phase(
            "default-active",
            active_duration,
            sample_interval,
            &mut app_session,
            |session| {
                let _ = session
                    .drain_pending_runtime_updates()
                    .expect("active drain should succeed");
            },
            &mut active_samples,
        );
        let _ = app_session
            .teardown()
            .expect("default app teardown should succeed");
        let app_metrics = app_session.debug_metrics();
        all_teardown_metrics.push(app_metrics.clone());

        let reload_deadline = Instant::now() + reload_duration;
        let mut reload_samples = Vec::new();
        while Instant::now() < reload_deadline {
            let (mut session, _rendered) =
                JsxRuntimeSession::load(&app_path).expect("reload soak session should load");
            let _ = session
                .drain_pending_runtime_updates()
                .expect("reload soak drain should succeed");
            reload_samples.push(sample_process("reload-phase", &session.debug_metrics()));
            let _ = session
                .teardown()
                .expect("reload soak teardown should succeed");
            let metrics = session.debug_metrics();
            reload_teardown_metrics.push(metrics.clone());
            all_teardown_metrics.push(metrics);
            std::thread::sleep(sample_interval);
        }

        assert!(
            !idle_samples.is_empty(),
            "soak harness should collect idle samples"
        );
        let idle_tail = &idle_samples[idle_samples.len() / 2..];
        let idle_avg_cpu = idle_tail
            .iter()
            .map(|sample| sample.cpu_percent)
            .sum::<f64>()
            / idle_tail.len().max(1) as f64;

        let mut all_samples = Vec::new();
        all_samples.extend(idle_samples);
        all_samples.extend(active_samples);
        all_samples.extend(reload_samples);
        let final_metrics = all_teardown_metrics
            .last()
            .cloned()
            .unwrap_or_else(|| app_metrics.clone());
        let mut gates = BTreeMap::new();
        gates.insert(
            "no_active_timers_after_teardown".to_owned(),
            all_teardown_metrics
                .iter()
                .all(|metrics| metrics.runtime.active_timer_count == 0),
        );
        gates.insert(
            "no_pending_wake_after_teardown".to_owned(),
            all_teardown_metrics
                .iter()
                .all(|metrics| !metrics.runtime.pending_host_wake),
        );
        gates.insert(
            "reload_recovery_clean".to_owned(),
            reload_teardown_metrics.iter().all(|metrics| {
                metrics.runtime.active_timer_count == 0 && !metrics.runtime.pending_host_wake
            }),
        );
        gates.insert(
            "memory_plateau".to_owned(),
            rss_plateau_within_band(&all_samples, rss_band_kb),
        );
        gates.insert(
            "idle_cpu_near_zero".to_owned(),
            idle_avg_cpu <= idle_cpu_threshold,
        );
        gates.insert(
            "timer_count_stable".to_owned(),
            !shows_monotonic_growth(
                all_samples
                    .iter()
                    .map(|sample| sample.metrics.runtime.active_timer_count),
            ),
        );
        gates.insert(
            "pending_wake_stable".to_owned(),
            !shows_monotonic_growth(
                all_samples
                    .iter()
                    .map(|sample| sample.metrics.runtime.pending_host_wake),
            ),
        );
        let artifact = ReadinessArtifact {
            git_sha: git_sha(),
            timestamp_unix_secs: unix_timestamp_secs(),
            scenario: "soak".to_owned(),
            command: command.to_owned(),
            passed: gates.values().all(|passed| *passed),
            gates,
            metrics: ReadinessArtifactMetrics {
                runtime: serde_json::to_value(&final_metrics.runtime)
                    .expect("runtime metrics should serialize"),
                bridge: final_metrics,
                host: ExampleHostMetrics::default(),
                extras: serde_json::json!({
                    "sample_interval_secs": sample_interval.as_secs_f64(),
                    "idle_duration_secs": idle_duration.as_secs_f64(),
                    "active_duration_secs": active_duration.as_secs_f64(),
                    "reload_duration_secs": reload_duration.as_secs_f64(),
                    "rss_band_kb": rss_band_kb,
                    "idle_cpu_threshold": idle_cpu_threshold,
                    "idle_avg_cpu": idle_avg_cpu,
                    "motion_session": motion_metrics,
                    "default_app_session": app_metrics,
                    "reload_teardown_count": reload_teardown_metrics.len(),
                    "all_teardown_count": all_teardown_metrics.len(),
                }),
            },
            samples: all_samples.clone(),
        };
        write_readiness_artifact("soak.json", &artifact, Some(&artifact.samples));
        assert!(
            artifact.passed,
            "soak readiness gates failed: {}",
            serde_json::to_string_pretty(&artifact)
                .unwrap_or_else(|error| format!("failed to serialize soak artifact: {error}"))
        );
    }

    #[derive(Debug, Clone, Default, serde::Serialize)]
    struct StressHarnessTotals {
        contract_tree_materialization_count: u64,
        contract_tree_noop_update_count: u64,
        motion_only_update_count: u64,
    }

    impl StressHarnessTotals {
        fn record(&mut self, metrics: &JsxRuntimeDebugMetrics) {
            self.contract_tree_materialization_count += metrics.contract_tree_materialization_count;
            self.contract_tree_noop_update_count += metrics.contract_tree_noop_update_count;
            self.motion_only_update_count += metrics.motion_only_update_count;
        }
    }

    #[derive(Debug, Clone, serde::Serialize)]
    struct ProcessSample {
        timestamp_unix_secs: u64,
        phase: String,
        rss_kb: u64,
        cpu_percent: f64,
        metrics: JsxRuntimeDebugMetrics,
    }

    #[derive(Debug, Clone, serde::Serialize)]
    struct ReadinessArtifact {
        git_sha: String,
        timestamp_unix_secs: u64,
        scenario: String,
        command: String,
        passed: bool,
        gates: BTreeMap<String, bool>,
        metrics: ReadinessArtifactMetrics,
        samples: Vec<ProcessSample>,
    }

    #[derive(Debug, Clone, serde::Serialize)]
    struct ReadinessArtifactMetrics {
        runtime: serde_json::Value,
        bridge: JsxRuntimeDebugMetrics,
        host: ExampleHostMetrics,
        extras: serde_json::Value,
    }

    fn write_stress_copy(path: &std::path::Path, value: &str) {
        std::fs::write(path, format!("export const stressCopy = \"{value}\";\n"))
            .expect("stress copy should be written");
    }

    fn read_env_u64(name: &str, default: u64) -> u64 {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(default)
    }

    fn read_env_f64(name: &str, default: f64) -> f64 {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(default)
    }

    fn sample_phase(
        phase: &str,
        duration: Duration,
        sample_interval: Duration,
        session: &mut JsxRuntimeSession,
        mut pump: impl FnMut(&mut JsxRuntimeSession),
        samples: &mut Vec<ProcessSample>,
    ) {
        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            pump(session);
            samples.push(sample_process(phase, &session.debug_metrics()));
            std::thread::sleep(sample_interval);
        }
    }

    fn sample_process(phase: &str, metrics: &JsxRuntimeDebugMetrics) -> ProcessSample {
        let pid = std::process::id().to_string();
        let rss_kb = command_output_value(&["-o", "rss=", "-p", pid.as_str()])
            .parse::<u64>()
            .unwrap_or_else(|error| panic!("failed to parse RSS sample: {error}"));
        let cpu_percent = command_output_value(&["-o", "%cpu=", "-p", pid.as_str()])
            .parse::<f64>()
            .unwrap_or_else(|error| panic!("failed to parse CPU sample: {error}"));
        ProcessSample {
            timestamp_unix_secs: unix_timestamp_secs(),
            phase: phase.to_owned(),
            rss_kb,
            cpu_percent,
            metrics: metrics.clone(),
        }
    }

    fn command_output_value(args: &[&str]) -> String {
        String::from_utf8(
            Command::new("ps")
                .args(args)
                .output()
                .expect("ps should run")
                .stdout,
        )
        .expect("ps output should be utf-8")
        .trim()
        .to_owned()
    }

    fn git_sha() -> String {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("git rev-parse should run");
        String::from_utf8(output.stdout)
            .expect("git sha output should be utf-8")
            .trim()
            .to_owned()
    }

    fn unix_timestamp_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_secs()
    }

    fn readiness_output_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("jsx-runtime-readiness")
    }

    fn write_readiness_artifact(
        file_name: &str,
        artifact: &ReadinessArtifact,
        samples: Option<&[ProcessSample]>,
    ) {
        let output_dir = readiness_output_dir();
        std::fs::create_dir_all(&output_dir).expect("readiness artifact dir should exist");
        let artifact_path = output_dir.join(file_name);
        let artifact_json =
            serde_json::to_string_pretty(artifact).expect("artifact json should serialize");
        std::fs::write(&artifact_path, artifact_json).expect("artifact json should be written");
        if let Some(samples) = samples {
            write_samples_csv(&output_dir.join("samples.csv"), &artifact.scenario, samples);
        }
    }

    fn write_samples_csv(path: &std::path::Path, scenario: &str, samples: &[ProcessSample]) {
        let mut csv = String::from(
            "scenario,timestamp_unix_secs,phase,rss_kb,cpu_percent,active_timer_count,pending_host_wake,contract_tree_materialization_count,contract_tree_noop_update_count,motion_only_update_count,host_wake_count,host_callbacks_invoked\n",
        );
        for sample in samples {
            csv.push_str(&format!(
                "{scenario},{},{},{},{:.2},{},{},{},{},{},{},{}\n",
                sample.timestamp_unix_secs,
                sample.phase,
                sample.rss_kb,
                sample.cpu_percent,
                sample.metrics.runtime.active_timer_count,
                sample.metrics.runtime.pending_host_wake,
                sample.metrics.contract_tree_materialization_count,
                sample.metrics.contract_tree_noop_update_count,
                sample.metrics.motion_only_update_count,
                sample.metrics.runtime.host_wake_count,
                sample.metrics.runtime.host_callbacks_invoked,
            ));
        }
        std::fs::write(path, csv).expect("samples csv should be written");
    }

    fn rss_plateau_within_band(samples: &[ProcessSample], band_kb: u64) -> bool {
        if samples.is_empty() {
            return false;
        }
        let warmup_index = samples.len() / 3;
        let steady_state = &samples[warmup_index..];
        if steady_state.is_empty() {
            return false;
        }
        let min_rss = steady_state
            .iter()
            .map(|sample| sample.rss_kb)
            .min()
            .expect("steady-state samples should exist");
        let max_rss = steady_state
            .iter()
            .map(|sample| sample.rss_kb)
            .max()
            .expect("steady-state samples should exist");
        let final_sample = steady_state
            .last()
            .expect("steady-state samples should have a final element");
        max_rss.saturating_sub(min_rss) <= band_kb
            && final_sample.metrics.runtime.active_timer_count
                <= final_sample.metrics.runtime.active_timer_high_water
    }

    fn shows_monotonic_growth<T>(values: impl IntoIterator<Item = T>) -> bool
    where
        T: Copy + PartialOrd,
    {
        let mut previous = None;
        let mut increased = false;
        for value in values {
            if let Some(previous_value) = previous {
                if value < previous_value {
                    return false;
                }
                if value > previous_value {
                    increased = true;
                }
            }
            previous = Some(value);
        }
        increased
    }

    fn checkbox_value(node: &ContractNode, node_id: &str) -> Option<bool> {
        if node.node_id().as_str() == node_id {
            if let ContractNode::Checkbox(checkbox) = node {
                return Some(checkbox.value);
            }
        }

        for child in contract_children(node) {
            if let Some(value) = checkbox_value(child, node_id) {
                return Some(value);
            }
        }

        None
    }

    fn input_value<'a>(node: &'a ContractNode, node_id: &str) -> Option<&'a str> {
        if node.node_id().as_str() == node_id {
            if let ContractNode::Input(input) = node {
                return Some(input.value.as_str());
            }
        }

        for child in contract_children(node) {
            if let Some(value) = input_value(child, node_id) {
                return Some(value);
            }
        }

        None
    }

    fn number_input_value(node: &ContractNode, node_id: &str) -> Option<f32> {
        if node.node_id().as_str() == node_id {
            if let ContractNode::NumberInput(input) = node {
                return Some(input.value);
            }
        }

        for child in contract_children(node) {
            if let Some(value) = number_input_value(child, node_id) {
                return Some(value);
            }
        }

        None
    }

    fn slider_value(node: &ContractNode, node_id: &str) -> Option<f32> {
        if node.node_id().as_str() == node_id {
            if let ContractNode::Slider(slider) = node {
                return Some(slider.value);
            }
        }

        for child in contract_children(node) {
            if let Some(value) = slider_value(child, node_id) {
                return Some(value);
            }
        }

        None
    }

    fn switch_value(node: &ContractNode, node_id: &str) -> Option<bool> {
        if node.node_id().as_str() == node_id {
            if let ContractNode::Switch(toggle) = node {
                return Some(toggle.value);
            }
        }

        for child in contract_children(node) {
            if let Some(value) = switch_value(child, node_id) {
                return Some(value);
            }
        }

        None
    }

    fn radio_value(node: &ContractNode, node_id: &str) -> Option<bool> {
        if node.node_id().as_str() == node_id {
            if let ContractNode::Radio(radio) = node {
                return Some(radio.value);
            }
        }

        for child in contract_children(node) {
            if let Some(value) = radio_value(child, node_id) {
                return Some(value);
            }
        }

        None
    }

    fn find_node<'a>(node: &'a ContractNode, node_id: &str) -> Option<&'a ContractNode> {
        if node.node_id().as_str() == node_id {
            return Some(node);
        }

        for child in contract_children(node) {
            if let Some(found) = find_node(child, node_id) {
                return Some(found);
            }
        }

        None
    }

    fn contract_children(node: &ContractNode) -> &[ContractNode] {
        match node {
            ContractNode::Row(props) => &props.children,
            ContractNode::Column(props) => &props.children,
            ContractNode::Inset(props) => &props.children,
            ContractNode::SizedBox(props) => &props.children,
            ContractNode::Card(props) => &props.children,
            ContractNode::Sidebar(props) => &props.children,
            ContractNode::Toolbar(props) => &props.children,
            ContractNode::Collapsible(props) => &props.children,
            ContractNode::DialogueModal(props) => &props.children,
            ContractNode::Popover(props) => &props.children,
            ContractNode::ContextMenu(props) => &props.children,
            ContractNode::AudioPlayback(props) => &props.children,
            ContractNode::ImageTile(props) => &props.children,
            _ => &[],
        }
    }

    fn label_text(node: &ContractNode) -> Option<&str> {
        match node {
            ContractNode::Label(props) => Some(props.text.as_str()),
            _ => None,
        }
    }

    fn common_class(node: &ContractNode) -> Option<&str> {
        node.common().class.as_deref()
    }

    fn button_variant(node: &ContractNode) -> Option<String> {
        match node {
            ContractNode::Button(props) => props.variant.map(|variant| format!("{variant:?}")),
            _ => None,
        }
    }

    fn button_selected(node: &ContractNode) -> Option<bool> {
        match node {
            ContractNode::Button(props) => Some(props.selected),
            _ => None,
        }
    }

    fn button_label(node: &ContractNode) -> Option<&str> {
        match node {
            ContractNode::Button(props) => Some(props.label.as_str()),
            _ => None,
        }
    }

    fn drain_async_until_update(
        session: &mut JsxRuntimeSession,
        timeout: Duration,
    ) -> Option<clay_jsx_egui_bridge::RenderedJsx> {
        let deadline = Instant::now() + timeout;
        loop {
            match session
                .drain_pending_runtime_updates()
                .expect("draining runtime updates should succeed")
            {
                Some(rendered) => return Some(rendered),
                None if Instant::now() >= deadline => return None,
                None => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    }

    fn drain_async_until(
        session: &mut JsxRuntimeSession,
        timeout: Duration,
        predicate: impl Fn(&clay_jsx_egui_bridge::RenderedJsx) -> bool,
    ) -> Option<clay_jsx_egui_bridge::RenderedJsx> {
        let deadline = Instant::now() + timeout;
        loop {
            match session
                .drain_pending_runtime_updates()
                .expect("draining runtime updates should succeed")
            {
                Some(rendered) if predicate(&rendered) => return Some(rendered),
                Some(_) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                Some(_) => return None,
                None if Instant::now() >= deadline => return None,
                None => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    }

    fn assert_motion_close(
        frame: &MotionFrame,
        node_id: &str,
        property: MotionProperty,
        expected: f32,
    ) {
        let actual = frame
            .values
            .get(&NodeId::from(node_id))
            .and_then(|values| values.get(property))
            .unwrap_or_else(|| panic!("missing motion value {property:?} on node {node_id}"));
        assert!(
            (actual - expected).abs() <= 0.001,
            "expected {property:?} on {node_id} to be close to {expected}, got {actual}"
        );
    }
}
