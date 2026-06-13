use std::collections::BTreeMap;
use std::collections::HashMap;
use zellij_tile::prelude::actions::Action;
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    tab_count: usize,
    active_tab: usize,
    active_tab_name: String,
    mode: InputMode,
    all_hints: HashMap<String, Vec<(String, String)>>,
    hints: Vec<(String, String)>,
}

register_plugin!(State);

fn key_label(key: &KeyWithModifier) -> String {
    let modifier = if key.key_modifiers.contains(&KeyModifier::Ctrl) {
        "Ctrl+"
    } else if key.key_modifiers.contains(&KeyModifier::Alt) {
        "Alt+"
    } else {
        ""
    };

    let k = match &key.bare_key {
        BareKey::Char(c) => c.to_string(),
        BareKey::Left => "←".to_string(),
        BareKey::Right => "→".to_string(),
        BareKey::Up => "↑".to_string(),
        BareKey::Down => "↓".to_string(),
        BareKey::Enter => "Enter".to_string(),
        BareKey::Esc => "Esc".to_string(),
        BareKey::Tab => "Tab".to_string(),
        BareKey::Backspace => "⌫".to_string(),
        BareKey::PageUp => "PgUp".to_string(),
        BareKey::PageDown => "PgDn".to_string(),
        _ => format!("{:?}", key.bare_key),
    };

    format!("{}{}", modifier, k)
}

fn action_label(action: &Action) -> Option<String> {
    match action {
        Action::NewTab { .. } => Some("new tab".into()),
        Action::CloseTab => Some("close tab".into()),
        Action::GoToNextTab => Some("next tab".into()),
        Action::GoToPreviousTab => Some("prev tab".into()),
        Action::GoToTab { index, .. } => Some(format!("go to tab {}", index)),
        Action::NewPane { direction, .. } => match direction {
            Some(Direction::Down) => Some("new pane down".into()),
            Some(Direction::Right) => Some("new pane right".into()),
            _ => Some("new pane".into()),
        },
        Action::NewStackedPane { .. } => Some("new stacked pane".into()),
        Action::ToggleFocusFullscreen => Some("fullscreen".into()),
        Action::ToggleFloatingPanes => Some("floating pane".into()),
        Action::TogglePaneFrames => Some("frames".into()),
        Action::TogglePaneInGroup => Some("toggle group".into()),
        Action::ToggleGroupMarking => Some("mark group".into()),
        Action::TogglePanePinned => Some("pin pane".into()),
        Action::ToggleActiveSyncTab => Some("sync tabs".into()),
        Action::SwitchFocus => Some("switch focus".into()),
        Action::MoveFocus { direction } => match direction {
            Direction::Left => Some("focus left".into()),
            Direction::Right => Some("focus right".into()),
            Direction::Up => Some("focus up".into()),
            Direction::Down => Some("focus down".into()),
        },
        Action::MoveFocusOrTab { direction } => match direction {
            Direction::Left => Some("focus/tab left".into()),
            Direction::Right => Some("focus/tab right".into()),
            Direction::Up => Some("focus up".into()),
            Direction::Down => Some("focus down".into()),
        },
        Action::MoveTab { direction } => match direction {
            Direction::Left => Some("move tab left".into()),
            Direction::Right => Some("move tab right".into()),
            _ => None,
        },
        Action::MovePane { direction } => match direction {
            Some(Direction::Left) => Some("move pane left".into()),
            Some(Direction::Right) => Some("move pane right".into()),
            Some(Direction::Up) => Some("move pane up".into()),
            Some(Direction::Down) => Some("move pane down".into()),
            _ => Some("move pane".into()),
        },
        Action::Resize { direction, resize } => match resize {
            Resize::Increase => match direction {
                Some(d) => Some(format!("resize + {:?}", d).to_lowercase()),
                None => Some("resize +".into()),
            },
            Resize::Decrease => match direction {
                Some(d) => Some(format!("resize - {:?}", d).to_lowercase()),
                None => Some("resize -".into()),
            },
        },
        Action::PreviousSwapLayout => Some("prev layout".into()),
        Action::NextSwapLayout => Some("next layout".into()),
        Action::EditScrollback { .. } => Some("edit scrollback".into()),
        Action::ScrollUp => Some("scroll up".into()),
        Action::ScrollDown => Some("scroll down".into()),
        Action::ScrollToTop => Some("scroll top".into()),
        Action::ScrollToBottom => Some("scroll bottom".into()),
        Action::PageScrollUp => Some("page up".into()),
        Action::PageScrollDown => Some("page down".into()),
        Action::HalfPageScrollUp => Some("half page up".into()),
        Action::HalfPageScrollDown => Some("half page down".into()),
        Action::Detach => Some("detach".into()),
        Action::Quit => Some("quit".into()),
        Action::SwitchToMode { input_mode } => match input_mode {
            InputMode::Normal => Some("normal mode".into()),
            InputMode::Tab => Some("tab mode".into()),
            InputMode::Pane => Some("pane mode".into()),
            InputMode::Session => Some("session mode".into()),
            InputMode::Scroll => Some("scroll mode".into()),
            InputMode::Resize => Some("resize mode".into()),
            InputMode::Move => Some("move mode".into()),
            InputMode::Tmux => Some("tmux mode".into()),
            _ => None,
        },
        _ => None,
    }
}

fn build_hints(
    keybinds: &[(KeyWithModifier, Vec<Action>)],
    only_ctrl: bool,
) -> Vec<(String, String)> {
    keybinds
        .iter()
        .filter(|(key, _)| {
            if only_ctrl {
                key.key_modifiers.contains(&KeyModifier::Ctrl)
            } else {
                true
            }
        })
        .filter_map(|(key, actions)| {
            let label = actions.iter().find_map(|a| action_label(a))?;
            Some((key_label(key), label))
        })
        .collect()
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[PermissionType::ReadApplicationState]);
        subscribe(&[EventType::PermissionRequestResult]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                subscribe(&[EventType::TabUpdate, EventType::ModeUpdate]);
                set_selectable(false);
                true
            }
            Event::TabUpdate(tabs) => {
                self.tab_count = tabs.len();
                if let Some(active) = tabs.iter().find(|t| t.active) {
                    self.active_tab = tabs
                        .iter()
                        .position(|t| t.active)
                        .map(|i| i + 1)
                        .unwrap_or(1);
                    self.active_tab_name = if active.name.is_empty() {
                        format!("Tab {}", self.active_tab)
                    } else {
                        active.name.clone()
                    };
                }
                true
            }
            Event::ModeUpdate(mode_info) => {
                self.mode = mode_info.mode.clone();

                self.all_hints.clear();
                for (mode, keybinds) in &mode_info.keybinds {
                    let only_ctrl = matches!(mode, InputMode::Normal);
                    let hints = build_hints(keybinds, only_ctrl);
                    if !hints.is_empty() {
                        self.all_hints.insert(format!("{:?}", mode), hints);
                    }
                }

                let only_ctrl = matches!(self.mode, InputMode::Normal);
                let keybinds = mode_info
                    .keybinds
                    .iter()
                    .find(|(m, _)| m == &self.mode)
                    .map(|(_, k)| k.clone())
                    .unwrap_or_default();
                self.hints = build_hints(&keybinds, only_ctrl);

                true
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        let tabs = format!(
            " {}/{} {} ",
            self.active_tab, self.tab_count, self.active_tab_name
        );
        let hints_str: String = self
            .hints
            .iter()
            .take(6)
            .map(|(k, v)| format!(" {} {} ", k, v))
            .collect::<Vec<_>>()
            .join("│");

        print_text_with_coordinates(Text::new(&tabs), 0, 0, None, None);
        print_text_with_coordinates(Text::new(&hints_str), tabs.len(), 0, None, None);
    }
}
