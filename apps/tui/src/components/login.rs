use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::Action,
    config::Config,
    profile::{ProfileStorage, UserProfile},
};

#[derive(Debug, PartialEq, Eq)]
enum InputFocus {
    ProfileList,
    Username,
    Password,
}

#[derive(Debug, PartialEq, Eq)]
enum ViewMode {
    ProfileSelection,
    ManualInput,
}

pub struct Login {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    username: String,
    password: String,
    focus: InputFocus,
    error_message: Option<String>,
    profiles: Vec<UserProfile>,
    selected_profile_index: usize,
    view_mode: ViewMode,
}

impl Default for Login {
    fn default() -> Self {
        // Load saved profiles
        let profiles = ProfileStorage::load()
            .ok()
            .and_then(|storage| {
                let profiles = storage.get_profiles().to_vec();
                if profiles.is_empty() {
                    None
                } else {
                    Some(profiles)
                }
            })
            .unwrap_or_default();

        // Determine view mode based on whether we have profiles
        let (view_mode, focus) = if profiles.is_empty() {
            (ViewMode::ManualInput, InputFocus::Username)
        } else {
            (ViewMode::ProfileSelection, InputFocus::ProfileList)
        };

        Self {
            command_tx: None,
            config: Config::default(),
            username: String::new(),
            password: String::new(),
            focus,
            error_message: None,
            profiles,
            selected_profile_index: 0,
            view_mode,
        }
    }
}

impl Login {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for Login {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FocusUsername => {
                self.focus = InputFocus::Username;
            }
            Action::FocusPassword => {
                self.focus = InputFocus::Password;
            }
            Action::LoginFailed { error } => {
                self.error_message = Some(error);
                self.password.clear();
                self.username.clear();
                // Reset to profile list if we have profiles
                if !self.profiles.is_empty() {
                    self.view_mode = ViewMode::ProfileSelection;
                    self.focus = InputFocus::ProfileList;
                    self.selected_profile_index = 0;
                } else {
                    self.view_mode = ViewMode::ManualInput;
                    self.focus = InputFocus::Username;
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match self.view_mode {
            ViewMode::ProfileSelection => {
                match key.code {
                    KeyCode::Up => {
                        if self.selected_profile_index > 0 {
                            self.selected_profile_index -= 1;
                        }
                        self.error_message = None;
                    }
                    KeyCode::Down => {
                        // profiles.len() is the "New user" option
                        if self.selected_profile_index < self.profiles.len() {
                            self.selected_profile_index += 1;
                        }
                        self.error_message = None;
                    }
                    KeyCode::Enter => {
                        // Check if "New user" is selected (index == profiles.len())
                        if self.selected_profile_index == self.profiles.len() {
                            // Switch to manual input mode
                            self.view_mode = ViewMode::ManualInput;
                            self.focus = InputFocus::Username;
                            self.username.clear();
                            self.password.clear();
                        } else {
                            // Select existing profile and move to password
                            self.username =
                                self.profiles[self.selected_profile_index].username.clone();
                            self.view_mode = ViewMode::ManualInput;
                            self.focus = InputFocus::Password;
                        }
                        self.error_message = None;
                    }
                    KeyCode::Esc => {
                        // Allow user to go back to profile list if they're in password mode
                        if self.focus == InputFocus::Password {
                            self.view_mode = ViewMode::ProfileSelection;
                            self.focus = InputFocus::ProfileList;
                            self.password.clear();
                        }
                    }
                    _ => {}
                }
            }
            ViewMode::ManualInput => {
                match key.code {
                    KeyCode::Tab => {
                        self.focus = match self.focus {
                            InputFocus::Username => InputFocus::Password,
                            InputFocus::Password => InputFocus::Username,
                            InputFocus::ProfileList => InputFocus::Username,
                        };
                    }
                    KeyCode::Enter => {
                        if !self.username.is_empty() && !self.password.is_empty() {
                            return Ok(Some(Action::Login {
                                username: self.username.clone(),
                                password: self.password.clone(),
                            }));
                        }
                    }
                    KeyCode::Char(c) => {
                        match self.focus {
                            InputFocus::Username => self.username.push(c),
                            InputFocus::Password => self.password.push(c),
                            InputFocus::ProfileList => {}
                        }
                        self.error_message = None;
                    }
                    KeyCode::Backspace => match self.focus {
                        InputFocus::Username => {
                            self.username.pop();
                        }
                        InputFocus::Password => {
                            self.password.pop();
                        }
                        InputFocus::ProfileList => {}
                    },
                    KeyCode::Esc => {
                        // Go back to profile selection if we have profiles
                        if !self.profiles.is_empty() {
                            self.view_mode = ViewMode::ProfileSelection;
                            self.focus = InputFocus::ProfileList;
                            self.username.clear();
                            self.password.clear();
                            self.selected_profile_index = 0;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Title
        let title = Paragraph::new("Scylla Login")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);

        match self.view_mode {
            ViewMode::ProfileSelection => {
                // Layout: Title, Profile List, Instructions
                let vertical = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        Constraint::Percentage(20),
                        Constraint::Length((self.profiles.len() as u16 + 3).min(12)),
                        Constraint::Length(3),
                        Constraint::Percentage(40),
                    ])
                    .split(area);

                frame.render_widget(title, vertical[0]);

                // Profile list with "New user" option
                let horizontal = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![
                        Constraint::Percentage(30),
                        Constraint::Percentage(40),
                        Constraint::Percentage(30),
                    ])
                    .split(vertical[1]);

                let mut profile_items: Vec<ListItem> = self
                    .profiles
                    .iter()
                    .enumerate()
                    .map(|(i, profile)| {
                        let content = if i == self.selected_profile_index {
                            format!("> {}", profile.username)
                        } else {
                            format!("  {}", profile.username)
                        };
                        ListItem::new(content)
                    })
                    .collect();

                // Add "New user" option
                let new_user_content = if self.selected_profile_index == self.profiles.len() {
                    "+ New user"
                } else {
                    "  New user"
                };
                profile_items.push(
                    ListItem::new(new_user_content).style(Style::default().fg(Color::DarkGray)),
                );

                let profiles_list = List::new(profile_items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Select Profile")
                        .border_style(Style::default().fg(Color::Yellow)),
                );

                frame.render_widget(profiles_list, horizontal[1]);

                // Instructions
                let instructions = if let Some(error) = &self.error_message {
                    Paragraph::new(error.as_str())
                        .style(Style::default().fg(Color::Red))
                        .alignment(Alignment::Center)
                } else {
                    Paragraph::new("↑↓: Navigate | Enter: Select | Ctrl+C: Quit")
                        .style(Style::default().fg(Color::Gray))
                        .alignment(Alignment::Center)
                };
                frame.render_widget(instructions, vertical[2]);
            }
            ViewMode::ManualInput => {
                // Layout: Title, Username, Password, Instructions
                let vertical = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        Constraint::Percentage(20),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Percentage(40),
                    ])
                    .split(area);

                frame.render_widget(title, vertical[0]);

                // Username input
                let horizontal = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![
                        Constraint::Percentage(30),
                        Constraint::Percentage(40),
                        Constraint::Percentage(30),
                    ])
                    .split(vertical[1]);

                let username_block = Block::default()
                    .borders(Borders::ALL)
                    .title("Username")
                    .border_style(if self.focus == InputFocus::Username {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    });
                let username_input = Paragraph::new(self.username.as_str()).block(username_block);
                frame.render_widget(username_input, horizontal[1]);

                // Password input
                let horizontal = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![
                        Constraint::Percentage(30),
                        Constraint::Percentage(40),
                        Constraint::Percentage(30),
                    ])
                    .split(vertical[2]);

                let password_block = Block::default()
                    .borders(Borders::ALL)
                    .title("Password")
                    .border_style(if self.focus == InputFocus::Password {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    });
                let masked_password = "*".repeat(self.password.len());
                let password_input = Paragraph::new(masked_password.as_str()).block(password_block);
                frame.render_widget(password_input, horizontal[1]);

                // Instructions
                let instructions = if let Some(error) = &self.error_message {
                    Paragraph::new(error.as_str())
                        .style(Style::default().fg(Color::Red))
                        .alignment(Alignment::Center)
                } else {
                    let help_text = if !self.profiles.is_empty() {
                        "Tab: Switch field | Enter: Login | Esc: Back to profiles | Ctrl+C: Quit"
                    } else {
                        "Tab: Switch field | Enter: Login | Ctrl+C: Quit"
                    };
                    Paragraph::new(help_text)
                        .style(Style::default().fg(Color::Gray))
                        .alignment(Alignment::Center)
                };
                frame.render_widget(instructions, vertical[3]);
            }
        }

        Ok(())
    }
}
