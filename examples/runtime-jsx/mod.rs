mod app;

use std::path::PathBuf;

use egui::{Context, ViewportBuilder};
use egui_component::theme::{self, BaseColor, ThemeMode, ThemeSpec};

pub use app::RuntimeJsxApp;

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
    use super::default_entry_path;
    use clay_jsx_egui_bridge::{JsxRuntimeSession, MotionFrame, MotionProperty};
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
