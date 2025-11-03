use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, sidebar::Sidebar};
use crate::{action::Action, config::Config};

pub struct MainView {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    sidebar: Sidebar,
    selected_organization: Option<(String, String)>, // (id, name)
    selected_project: Option<(String, String)>,      // (id, name)
}

impl Default for MainView {
    fn default() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            sidebar: Sidebar::new(),
            selected_organization: None,
            selected_project: None,
        }
    }
}

impl MainView {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for MainView {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx.clone());
        self.sidebar.register_action_handler(tx)?;
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config.clone();
        self.sidebar.register_config_handler(config)?;
        Ok(())
    }

    fn init(&mut self, area: layout::Size) -> Result<()> {
        self.sidebar.init(area)?;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match &action {
            Action::SelectOrganization {
                organization_id,
                organization_name,
            } => {
                self.selected_organization =
                    Some((organization_id.clone(), organization_name.clone()));
            }
            Action::SelectProject {
                project_id,
                project_name,
            } => {
                self.selected_project = Some((project_id.clone(), project_name.clone()));
            }
            _ => {}
        }
        // Forward to sidebar
        self.sidebar.update(action)?;
        Ok(None)
    }

    fn handle_events(&mut self, event: Option<crate::tui::Event>) -> Result<Option<Action>> {
        // Forward events to sidebar
        self.sidebar.handle_events(event)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Create the outer layout (horizontal split)
        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(area);

        // Create the inner layout for the right side (vertical split)
        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(outer_layout[1]);

        // Render sidebar on the left
        self.sidebar.draw(frame, outer_layout[0])?;

        // Render top-right panel - Organization Info
        let top_panel_block = Block::default()
            .title("Organization Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let org_content = if let Some((org_id, org_name)) = &self.selected_organization {
            format!(
                "Organization: {}\n\nID: {}\n\nClick on the organization in the sidebar to toggle projects",
                org_name, org_id
            )
        } else {
            "No organization selected\n\nClick on an organization in the sidebar to view details"
                .to_string()
        };

        let top_panel_content = Paragraph::new(org_content)
            .block(top_panel_block)
            .wrap(Wrap { trim: false })
            .style(if self.selected_organization.is_some() {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            });
        frame.render_widget(top_panel_content, inner_layout[0]);

        // Render bottom-right panel - Project Info
        let bottom_panel_block = Block::default()
            .title("Project Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));

        let project_content = if let Some((project_id, project_name)) = &self.selected_project {
            format!(
                "Project: {}\n\nID: {}\n\nClick on projects in the sidebar to view their details",
                project_name, project_id
            )
        } else {
            "No project selected\n\nExpand an organization and click on a project to view details"
                .to_string()
        };

        let bottom_panel_content = Paragraph::new(project_content)
            .block(bottom_panel_block)
            .wrap(Wrap { trim: false })
            .style(if self.selected_project.is_some() {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            });
        frame.render_widget(bottom_panel_content, inner_layout[1]);

        Ok(())
    }
}
