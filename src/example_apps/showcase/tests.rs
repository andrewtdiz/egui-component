use super::{
    collab_cursor_preview_anchor, collab_cursor_preview_position, configure_snapshot,
    install_context, render_snapshot_surface, sidebar_preview_toggle_rect, update, ShowcaseApp,
};
use crate::{ComponentKind, ThemeMode};
use egui::{pos2, vec2, CentralPanel, Context, RawInput, Rect};

#[test]
fn snapshot_surface_renders_representative_components_without_panic() {
    let context = Context::default();
    install_context(&context);

    for component in [
        ComponentKind::CanvaBackgrounds,
        ComponentKind::CanvaBrandKit,
        ComponentKind::CanvaEditImage,
        ComponentKind::CanvaPosition,
        ComponentKind::CollabCursor,
        ComponentKind::DragBoard,
        ComponentKind::EmojiSelector,
        ComponentKind::FileTree,
        ComponentKind::Sidebar,
        ComponentKind::Toast,
    ] {
        let mut app = ShowcaseApp::default();
        configure_snapshot(&mut app, component, ThemeMode::Dark);

        let _ = context.run(RawInput::default(), |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                render_snapshot_surface(&mut app, ui);
            });
        });
    }
}

#[test]
fn showcase_update_renders_narrow_layout_without_panic() {
    let context = Context::default();
    install_context(&context);
    let mut app = ShowcaseApp::default();
    app.selected_component = ComponentKind::Sidebar;

    let _ = context.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(720.0, 540.0))),
            ..Default::default()
        },
        |ctx| update(&mut app, ctx),
    );
}

#[test]
fn sidebar_preview_toggle_rect_tracks_host_edges() {
    let host_rect = Rect::from_min_size(pos2(40.0, 24.0), vec2(720.0, 360.0));
    let sidebar_width = 240.0;

    let left_closed = sidebar_preview_toggle_rect(host_rect, 0, 0.0, sidebar_width);
    let left_open = sidebar_preview_toggle_rect(host_rect, 0, 1.0, sidebar_width);
    let right_closed = sidebar_preview_toggle_rect(host_rect, 1, 0.0, sidebar_width);
    let right_open = sidebar_preview_toggle_rect(host_rect, 1, 1.0, sidebar_width);

    assert_eq!(left_closed.left(), host_rect.left() + 8.0);
    assert_eq!(left_open.left(), host_rect.left() + sidebar_width - 36.0);
    assert_eq!(right_closed.right(), host_rect.right() - 8.0);
    assert_eq!(right_open.right(), host_rect.right() - sidebar_width + 36.0);
    assert_eq!(left_closed.top(), host_rect.top() + 8.0);
    assert_eq!(right_closed.top(), host_rect.top() + 8.0);
}

#[test]
fn collab_cursor_preview_click_mapping_stays_within_host() {
    let host_rect = Rect::from_min_size(pos2(40.0, 24.0), vec2(300.0, 200.0));

    assert_eq!(
        collab_cursor_preview_anchor(host_rect, pos2(400.0, -10.0)),
        vec2(1.0, 0.0)
    );
    assert_eq!(
        collab_cursor_preview_position(host_rect, vec2(0.5, 0.25)),
        pos2(190.0, 74.0)
    );
}
