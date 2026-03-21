mod showcase_hot_runtime;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ShowcaseCommand {
    Showcase,
    HotShowcase,
    Chat,
}

fn main() -> egui_component::Result {
    match parse_command(std::env::args().skip(1))? {
        ShowcaseCommand::Showcase => egui_component::dev::run_showcase(),
        ShowcaseCommand::HotShowcase => showcase_hot_runtime::run_hot_showcase_window(),
        ShowcaseCommand::Chat => egui_component::dev::run_chat(),
    }
}

fn parse_command(
    args: impl IntoIterator<Item = String>,
) -> std::result::Result<ShowcaseCommand, egui_component::ComponentLibraryError> {
    let mut args = args.into_iter().collect::<Vec<_>>();
    while args.first().is_some_and(|arg| arg == "--") {
        args.remove(0);
    }

    match args.as_slice() {
        [] => Ok(ShowcaseCommand::Showcase),
        [command] if command == "hot" => Ok(ShowcaseCommand::HotShowcase),
        [command] if command == "chat" => Ok(ShowcaseCommand::Chat),
        [command, mode] if command == "chat" && mode == "hot" => Err(usage_error(
            "`hot` is only supported for the showcase window",
        )),
        _ => Err(usage_error(
            "usage: cargo showcase | cargo showcase -- hot | cargo showcase -- chat",
        )),
    }
}

fn usage_error(message: &str) -> egui_component::ComponentLibraryError {
    egui_component::ComponentLibraryError::Runtime(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{parse_command, ShowcaseCommand};

    #[test]
    fn defaults_to_static_showcase() {
        assert_eq!(
            parse_command(Vec::<String>::new()).unwrap(),
            ShowcaseCommand::Showcase
        );
    }

    #[test]
    fn accepts_hot_showcase_mode() {
        assert_eq!(
            parse_command(vec!["hot".to_owned()]).unwrap(),
            ShowcaseCommand::HotShowcase
        );
    }

    #[test]
    fn accepts_hot_showcase_mode_with_alias_separator() {
        assert_eq!(
            parse_command(vec!["--".to_owned(), "hot".to_owned()]).unwrap(),
            ShowcaseCommand::HotShowcase
        );
    }

    #[test]
    fn rejects_hot_chat_mode() {
        let error = parse_command(vec!["chat".to_owned(), "hot".to_owned()]).unwrap_err();
        assert!(error.to_string().contains("only supported"));
    }
}
