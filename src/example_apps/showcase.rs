use crate::catalog::{self, ComponentDefinition, ComponentGroup, ComponentKind};
use crate::layout;
use crate::prelude::*;
use crate::theme::{self, BaseColor, ColorRole, RadiusRole, ThemeMode, ThemeSpec};
use crate::ui::tokens;
use egui::{
    vec2, Align2, CentralPanel, Color32, CornerRadius, CursorIcon, Id, Layout, Rect, Response,
    ScrollArea, Sense, SidePanel, Stroke, TopBottomPanel, Ui, UiBuilder,
};

include!("showcase/data.rs");
include!("showcase/chrome.rs");
include!("showcase/previews.rs");
include!("showcase/helpers.rs");
include!("showcase/metadata.rs");

#[cfg(test)]
mod tests;
