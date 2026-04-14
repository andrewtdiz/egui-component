use egui::scroll_area::ScrollSource;
use egui::ScrollArea;

/// Local scroll-area behavior for this project.
///
/// Egui enables drag-to-scroll by default. We turn that off everywhere so scroll
/// areas respond to wheel, trackpad, and scrollbar input without treating a
/// mouse-down drag on the contents as a scroll gesture.
pub trait ScrollAreaExt {
    fn no_drag_to_scroll(self) -> Self;
}

impl ScrollAreaExt for ScrollArea {
    #[inline]
    fn no_drag_to_scroll(self) -> Self {
        self.scroll_source(ScrollSource {
            drag: false,
            ..ScrollSource::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ScrollAreaExt;
    use egui::ScrollArea;

    #[test]
    fn no_drag_to_scroll_disables_content_dragging() {
        let debug = format!("{:?}", ScrollArea::vertical().no_drag_to_scroll());
        assert!(
            debug.contains("scroll_source: ScrollSource {"),
            "unexpected debug output: {debug}"
        );
        assert!(
            debug.contains("drag: false"),
            "unexpected debug output: {debug}"
        );
        assert!(
            debug.contains("mouse_wheel: true"),
            "unexpected debug output: {debug}"
        );
    }
}
