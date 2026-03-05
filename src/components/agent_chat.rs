use super::{
    button, card, label, separator, ButtonProps, ButtonVariant, CardProps, LabelProps, LabelTone,
};
use egui::{Layout, Ui};

#[derive(Debug, Clone, Copy)]
pub struct AgentChatProps<'a> {
    pub placeholder: &'a str,
    pub mode_label: &'a str,
    pub usage_label: &'a str,
}

impl<'a> AgentChatProps<'a> {
    pub fn new() -> Self {
        Self {
            placeholder: "Ask, Search or Chat...",
            mode_label: "Auto",
            usage_label: "52% used",
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn mode_label(mut self, mode_label: &'a str) -> Self {
        self.mode_label = mode_label;
        self
    }

    pub fn usage_label(mut self, usage_label: &'a str) -> Self {
        self.usage_label = usage_label;
        self
    }
}

impl Default for AgentChatProps<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AgentChatState {
    pub add_clicked: bool,
    pub send_clicked: bool,
}

pub fn agent_chat(ui: &mut Ui, props: AgentChatProps<'_>) -> AgentChatState {
    let mut state = AgentChatState::default();

    let _ = card(ui, CardProps::new(), |ui| {
        let _ = label(
            ui,
            LabelProps::new(props.placeholder).tone(LabelTone::Muted),
        );
        let _ = separator(ui);

        ui.horizontal(|ui| {
            if button(
                ui,
                ButtonProps::icon_only("plus").variant(ButtonVariant::Secondary),
            )
            .clicked()
            {
                state.add_clicked = true;
            }

            let _ = label(
                ui,
                LabelProps::new(props.mode_label).tone(LabelTone::Secondary),
            );

            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if button(ui, ButtonProps::icon_only("arrow-up")).clicked() {
                    state.send_clicked = true;
                }

                let _ = label(
                    ui,
                    LabelProps::new(props.usage_label).tone(LabelTone::Muted),
                );
            });
        });
    });

    state
}
