use std::path::PathBuf;

use crate::{
    components::{Card, ComponentUiExt, Label, LabelTone, LabelWeight},
    theme::{self, ThemeMode, ThemeSpec},
    ComponentGroup, ComponentKind,
};
use egui::{CentralPanel, Context, Layout, ScrollArea, SidePanel, TopBottomPanel, Ui};

use super::{runtime_egui_host, showcase};

pub const WINDOW_TITLE: &str = "egui-component Luau Component Gallery";
pub const WINDOW_INNER_SIZE: [f32; 2] = [1360.0, 940.0];

const GALLERY_COMPONENTS: &[ComponentKind] = &[
    ComponentKind::Button,
    ComponentKind::ButtonGroup,
    ComponentKind::Card,
    ComponentKind::Checkbox,
    ComponentKind::DropdownMenu,
    ComponentKind::Input,
    ComponentKind::Label,
    ComponentKind::NumberInput,
    ComponentKind::Progress,
    ComponentKind::Radio,
    ComponentKind::Select,
    ComponentKind::Separator,
    ComponentKind::Skeleton,
    ComponentKind::Slider,
    ComponentKind::Spinner,
    ComponentKind::Switch,
    ComponentKind::Tabs,
    ComponentKind::Tooltip,
    ComponentKind::Collapsible,
    ComponentKind::Field,
    ComponentKind::Pagination,
    ComponentKind::RadioGroup,
];

struct GalleryPreview {
    kind: ComponentKind,
    script_path: PathBuf,
    host: Option<runtime_egui_host::RuntimeEguiHostApp>,
}

impl GalleryPreview {
    fn new(kind: ComponentKind) -> Self {
        Self {
            kind,
            script_path: preview_script_path(kind),
            host: None,
        }
    }

    fn host(&mut self) -> &mut runtime_egui_host::RuntimeEguiHostApp {
        self.host.get_or_insert_with(|| {
            runtime_egui_host::RuntimeEguiHostApp::new_presentational(self.script_path.clone())
        })
    }
}

pub struct ComponentGalleryRuntimeApp {
    selected_component: ComponentKind,
    theme_mode: ThemeMode,
    previews: Vec<GalleryPreview>,
}

impl Default for ComponentGalleryRuntimeApp {
    fn default() -> Self {
        let mut previews = GALLERY_COMPONENTS
            .iter()
            .copied()
            .map(GalleryPreview::new)
            .collect::<Vec<_>>();
        previews.sort_by_key(|preview| {
            let definition = showcase::catalog_component_definition(preview.kind);
            (gallery_group_sort_key(definition.group), definition.label)
        });

        Self {
            selected_component: ComponentKind::Button,
            theme_mode: ThemeMode::System,
            previews,
        }
    }
}

impl ComponentGalleryRuntimeApp {
    fn render_sidebar(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);

        let _ = showcase::show_column(ui, 8.0, |ui| {
            let mut components = ui.components();
            let _ = components.label(
                Label::new("Components")
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Primary),
            );
        });

        ui.add_space(8.0);
        let _ = ui.components().separator();
        ui.add_space(8.0);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for group in [ComponentGroup::Primitive, ComponentGroup::Composed] {
                    let kinds = self.preview_kinds_for_group(group);
                    if kinds.is_empty() {
                        continue;
                    }

                    let _ = ui.components().label(
                        Label::new(gallery_group_title(group))
                            .tone(LabelTone::Muted)
                            .size(showcase::SMALL_TEXT)
                            .weight(LabelWeight::Semibold),
                    );
                    ui.add_space(4.0);

                    for kind in kinds {
                        let definition = showcase::catalog_component_definition(kind);
                        let selected = self.selected_component == kind;
                        if showcase::draw_showcase_sidebar_item(ui, definition.label, selected)
                            .clicked()
                        {
                            self.selected_component = kind;
                        }
                    }

                    ui.add_space(8.0);
                }
            });
    }

    fn render_center(&mut self, ctx: &Context, ui: &mut Ui) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let _ = ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                    self.render_preview_surface(ctx, ui);
                });
            });
    }

    fn render_preview_surface(&mut self, ctx: &Context, ui: &mut Ui) {
        let component = self.selected_component;
        let definition = showcase::catalog_component_definition(component);
        let width = showcase::preview_surface_width(component, ui.available_width());

        let _ = ui.components().card(Card::new().padding(14, 14), |ui| {
            ui.set_width(width);

            let mut components = ui.components();
            let _ = components.label(
                Label::new(definition.label)
                    .weight(LabelWeight::Semibold)
                    .tone(LabelTone::Primary),
            );
            let _ = components.label(
                Label::new(showcase::showcase_description(component))
                    .tone(LabelTone::Muted)
                    .size(showcase::SMALL_TEXT),
            );
            ui.add_space(8.0);
            let _ = ui.components().separator();
            ui.add_space(10.0);

            let host = self.selected_host_mut();
            host.process_frame_boundary(ctx);
            host.render_script_surface(ui);
        });
    }

    fn preview_kinds_for_group(&self, group: ComponentGroup) -> Vec<ComponentKind> {
        self.previews
            .iter()
            .filter_map(|preview| {
                let definition = showcase::catalog_component_definition(preview.kind);
                (definition.group == group).then_some(preview.kind)
            })
            .collect()
    }

    fn selected_host_mut(&mut self) -> &mut runtime_egui_host::RuntimeEguiHostApp {
        self.previews
            .iter_mut()
            .find(|preview| preview.kind == self.selected_component)
            .expect("missing component gallery preview host")
            .host()
    }
}

pub fn install_context(ctx: &Context) {
    showcase::install_context(ctx);
}

pub fn update(app: &mut ComponentGalleryRuntimeApp, ctx: &Context) {
    theme::set_theme(ctx, ThemeSpec::preset(showcase::SHOWCASE_BASE_COLOR));
    theme::set_mode(ctx, app.theme_mode);
    let runtime = theme::runtime_for_context(ctx);

    #[allow(
        deprecated,
        reason = "eframe App::update still renders top-level panels from Context"
    )]
    {
        TopBottomPanel::top("component_gallery_showcase_topbar")
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(showcase::showcase_header_fill(runtime))
                    .stroke(egui::Stroke::NONE),
            )
            .show(ctx, |ui| {
                let _ = showcase::show_inset(ui, 16, 10, |ui| {
                    showcase::show_showcase_topbar_row(
                        ui,
                        "egui-component Luau Component Gallery",
                        &mut app.theme_mode,
                    );
                });
                let _ = ui.components().separator();
            });

        SidePanel::left("component_gallery_showcase_sidebar")
            .resizable(true)
            .default_width(showcase::SIDEBAR_WIDTH)
            .min_width(200.0)
            .max_width(320.0)
            .show(ctx, |ui| app.render_sidebar(ui));

        CentralPanel::default().show(ctx, |ui| app.render_center(ctx, ui));
    }

    runtime_egui_host::finish_embedded_frame(app.selected_host_mut(), ctx);
}

fn gallery_group_sort_key(group: ComponentGroup) -> usize {
    match group {
        ComponentGroup::Primitive => 0,
        ComponentGroup::Composed => 1,
    }
}

fn gallery_group_title(group: ComponentGroup) -> &'static str {
    match group {
        ComponentGroup::Primitive => "Primary / Primitive",
        ComponentGroup::Composed => "Derived / Composed",
    }
}

fn preview_script_path(kind: ComponentKind) -> PathBuf {
    let definition = showcase::catalog_component_definition(kind);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("runtime-luau")
        .join("ui")
        .join("components")
        .join(format!("{}.luau", definition.id))
}

#[cfg(test)]
mod tests {
    use super::{install_context, update, ComponentGalleryRuntimeApp};
    use crate::ComponentKind;
    use egui::{
        epaint::Shape, pos2, vec2, Context, Event, FullOutput, Modifiers, PointerButton, Pos2,
        RawInput, Rect,
    };

    #[test]
    fn gallery_runtime_renders_reference_showcase_shell() {
        let context = Context::default();
        install_context(&context);
        let mut app = ComponentGalleryRuntimeApp::default();

        let rendered = run_frame(&context, &mut app);

        assert!(rendered
            .iter()
            .any(|text| text == "egui-component Luau Component Gallery"));
        assert!(rendered.iter().any(|text| text == "Primary / Primitive"));
        assert!(rendered.iter().any(|text| text == "Button"));
        assert!(rendered.iter().any(|text| text == "Button Group"));
    }

    #[test]
    fn gallery_runtime_sidebar_switches_selected_preview() {
        let context = Context::default();
        install_context(&context);
        let mut app = ComponentGalleryRuntimeApp::default();

        run_frame(&context, &mut app);

        let checkbox_center = find_text_center(
            &run_frame_output(&context, &mut app, RawInput::default()).shapes,
            "Checkbox",
        )
        .expect("expected Checkbox sidebar item");

        run_frame_output(&context, &mut app, pointer_input(checkbox_center, true));
        let clicked = collect_rendered_texts(
            &run_frame_output(&context, &mut app, pointer_input(checkbox_center, false)).shapes,
        );
        let post_click = run_frame(&context, &mut app);

        assert!(
            clicked.iter().any(|text| text == "Checkbox")
                || post_click.iter().any(|text| text == "Checkbox")
        );
        assert!(post_click
            .iter()
            .any(|text| text == "Receive Shadows"),
            "expected checkbox preview after sidebar selection; clicked={clicked:?} post_click={post_click:?}"
        );
    }

    #[test]
    fn tabs_preview_matches_reference_variant_stack() {
        let context = Context::default();
        install_context(&context);
        let mut app = ComponentGalleryRuntimeApp {
            selected_component: ComponentKind::Tabs,
            ..Default::default()
        };

        let rendered = run_frame(&context, &mut app);

        assert!(rendered.iter().any(|text| text == "Design"));
        assert!(rendered.iter().any(|text| text == "Layout"));
        assert!(rendered.iter().any(|text| text == "Templates"));
        assert!(rendered.iter().any(|text| text == "Home"));
    }

    #[test]
    fn gallery_previews_expose_reference_copy_for_each_component() {
        let context = Context::default();
        install_context(&context);
        let cases: &[(ComponentKind, &[&str])] = &[
            (
                ComponentKind::Label,
                &["Primary label", "Secondary label", "Destructive text"],
            ),
            (ComponentKind::Input, &["Player_Robot", "Robot"]),
            (
                ComponentKind::Field,
                &["Material", "Assigned material for selected mesh"],
            ),
            (ComponentKind::Button, &["Primary", "New Draft", "Export"]),
            (ComponentKind::ButtonGroup, &["Move", "Rotate", "Scale"]),
            (ComponentKind::Card, &["Card Title", "Save", "Cancel"]),
            (ComponentKind::Checkbox, &["Receive Shadows"]),
            (
                ComponentKind::Switch,
                &["Enable Post FX", "Use Compact Handles"],
            ),
            (ComponentKind::Slider, &["62"]),
            (ComponentKind::NumberInput, &["X", "Y"]),
            (ComponentKind::Select, &["Review"]),
            (
                ComponentKind::Tabs,
                &["Design", "Layout", "Templates", "Home"],
            ),
            (
                ComponentKind::Separator,
                &["Above separator", "Below separator"],
            ),
            (ComponentKind::Progress, &["Advance", "Reset"]),
            (
                ComponentKind::Spinner,
                &["Loading button example", "Publish"],
            ),
            (ComponentKind::Skeleton, &["Show Loaded State"]),
            (ComponentKind::Radio, &["Use publish channel", "Reset"]),
            (
                ComponentKind::RadioGroup,
                &["Starter", "Team", "Enterprise", "Clear"],
            ),
            (ComponentKind::Tooltip, &["Top", "Right", "Bottom", "Left"]),
            (
                ComponentKind::DropdownMenu,
                &["Open", "No action triggered"],
            ),
            (ComponentKind::Collapsible, &["Transform"]),
            (
                ComponentKind::Pagination,
                &["Previous", "Next", "Page 2 of 12"],
            ),
        ];

        for (kind, expected_texts) in cases {
            let mut app = ComponentGalleryRuntimeApp {
                selected_component: *kind,
                ..Default::default()
            };
            let rendered = run_frame(&context, &mut app);
            for expected in *expected_texts {
                assert!(
                    rendered.iter().any(|text| text == expected),
                    "expected `{expected}` in preview for {:?}; rendered={rendered:?}",
                    kind
                );
            }
        }
    }

    fn run_frame(context: &Context, app: &mut ComponentGalleryRuntimeApp) -> Vec<String> {
        let output = run_frame_output(context, app, RawInput::default());
        collect_rendered_texts(&output.shapes)
    }

    fn run_frame_output(
        context: &Context,
        app: &mut ComponentGalleryRuntimeApp,
        mut input: RawInput,
    ) -> FullOutput {
        input.screen_rect = Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1360.0, 940.0)));
        context.run(input, |ctx| update(app, ctx))
    }

    fn collect_rendered_texts(shapes: &[egui::epaint::ClippedShape]) -> Vec<String> {
        let mut rendered = Vec::new();
        for shape in shapes {
            collect_texts_from_shape(&shape.shape, &mut rendered);
        }
        rendered
    }

    fn collect_texts_from_shape(shape: &Shape, rendered: &mut Vec<String>) {
        match shape {
            Shape::Text(text) => rendered.push(text.galley.job.text.clone()),
            Shape::Vec(shapes) => {
                for shape in shapes {
                    collect_texts_from_shape(shape, rendered);
                }
            }
            _ => {}
        }
    }

    fn find_text_center(shapes: &[egui::epaint::ClippedShape], target: &str) -> Option<Pos2> {
        let mut text_rect = Rect::NOTHING;
        for clipped_shape in shapes {
            collect_text_rect(&clipped_shape.shape, target, &mut text_rect);
        }
        text_rect.is_positive().then(|| text_rect.center())
    }

    fn collect_text_rect(shape: &Shape, target: &str, text_rect: &mut Rect) {
        match shape {
            Shape::Text(text_shape) if text_shape.galley.job.text.contains(target) => {
                *text_rect = text_rect.union(text_shape.visual_bounding_rect());
            }
            Shape::Vec(shapes) => {
                for shape in shapes {
                    collect_text_rect(shape, target, text_rect);
                }
            }
            _ => {}
        }
    }

    fn pointer_input(position: Pos2, pressed: bool) -> RawInput {
        RawInput {
            events: vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            ..Default::default()
        }
    }
}
