use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashSet;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config};

#[derive(Debug, Clone)]
struct Organization {
    id: String,
    name: String,
    projects: Vec<Project>,
}

#[derive(Debug, Clone)]
struct Project {
    id: String,
    name: String,
}

pub struct Sidebar {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    organizations: Vec<Organization>,
    expanded_orgs: HashSet<String>,
    selected_index: usize,
    loading: bool,
    last_render_area: Option<Rect>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            organizations: Vec::new(),
            expanded_orgs: HashSet::new(),
            selected_index: 0,
            loading: true,
            last_render_area: None,
        }
    }
}

impl Sidebar {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_flat_items(&self) -> Vec<(String, usize, bool)> {
        // Returns: (display_text, indent_level, is_project)
        let mut items = Vec::new();
        for org in &self.organizations {
            items.push((format!("▶ {}", org.name), 0, false));
            if self.expanded_orgs.contains(&org.id) {
                for project in &org.projects {
                    items.push((format!("  └─ {}", project.name), 1, true));
                }
            }
        }
        items
    }

    fn get_total_items(&self) -> usize {
        self.get_flat_items().len()
    }
}

impl Component for Sidebar {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn init(&mut self, _area: layout::Size) -> Result<()> {
        // Request organizations on init
        if let Some(tx) = &self.command_tx {
            tx.send(Action::LoadOrganizations)?;
        }
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::OrganizationsLoaded { organizations } => {
                self.organizations = organizations
                    .into_iter()
                    .map(|(id, name)| Organization {
                        id,
                        name,
                        projects: Vec::new(),
                    })
                    .collect();
                self.loading = false;
            }
            Action::ProjectsLoaded {
                organization_id,
                projects,
            } => {
                if let Some(org) = self
                    .organizations
                    .iter_mut()
                    .find(|o| o.id == organization_id)
                {
                    org.projects = projects
                        .into_iter()
                        .map(|(id, name)| Project { id, name })
                        .collect();
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                let total = self.get_total_items();
                if total > 0 {
                    self.selected_index = (self.selected_index + 1).min(total - 1);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let items = self.get_flat_items();
                if let Some((text, _, is_project)) = items.get(self.selected_index) {
                    if !is_project {
                        // Toggle organization expansion
                        let org_name = text.trim_start_matches("▶ ");
                        if let Some(org) = self.organizations.iter().find(|o| o.name == org_name) {
                            let org_id = org.id.clone();
                            if self.expanded_orgs.contains(&org_id) {
                                self.expanded_orgs.remove(&org_id);
                            } else {
                                self.expanded_orgs.insert(org_id.clone());
                                // Load projects if not already loaded
                                if org.projects.is_empty() {
                                    return Ok(Some(Action::LoadProjects {
                                        organization_id: org_id,
                                    }));
                                }
                            }
                        }
                    } else {
                        // Select project
                        // Find which project was selected
                        for org in &self.organizations {
                            if self.expanded_orgs.contains(&org.id) {
                                for project in &org.projects {
                                    if text.contains(&project.name) {
                                        return Ok(Some(Action::SelectProject {
                                            project_id: project.id.clone(),
                                            project_name: project.name.clone(),
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        // Only handle left clicks
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return Ok(None);
        }

        // Check if click is within our sidebar area
        if let Some(area) = self.last_render_area {
            let click_x = mouse.column;
            let click_y = mouse.row;

            // Check if click is within the sidebar bounds
            if click_x < area.x
                || click_x >= area.x + area.width
                || click_y < area.y
                || click_y >= area.y + area.height
            {
                return Ok(None);
            }

            // Account for the block borders (1 line for top border, 1 line for title)
            let content_start_y = area.y + 2;
            let relative_y = click_y.saturating_sub(content_start_y);

            // Map the click to an item in the list
            let items = self.get_flat_items();
            if let Some((text, _, is_project)) = items.get(relative_y as usize) {
                if !is_project {
                    // Clicked on organization - toggle expansion
                    let org_name = text.trim_start_matches("▶ ");
                    if let Some(org) = self.organizations.iter().find(|o| o.name == org_name) {
                        let org_id = org.id.clone();
                        let org_name = org.name.clone();

                        if self.expanded_orgs.contains(&org_id) {
                            self.expanded_orgs.remove(&org_id);
                        } else {
                            self.expanded_orgs.insert(org_id.clone());
                            // Load projects if not already loaded
                            if org.projects.is_empty() {
                                return Ok(Some(Action::LoadProjects {
                                    organization_id: org_id.clone(),
                                }));
                            }
                        }

                        // Also send SelectOrganization action
                        return Ok(Some(Action::SelectOrganization {
                            organization_id: org_id,
                            organization_name: org_name,
                        }));
                    }
                } else {
                    // Clicked on project - select it
                    for org in &self.organizations {
                        if self.expanded_orgs.contains(&org.id) {
                            for project in &org.projects {
                                if text.contains(&project.name) {
                                    return Ok(Some(Action::SelectProject {
                                        project_id: project.id.clone(),
                                        project_name: project.name.clone(),
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Store the area for mouse event handling
        self.last_render_area = Some(area);
        let block = Block::default()
            .title("Organizations & Projects")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        if self.loading {
            let loading_text = Paragraph::new("Loading organizations...")
                .block(block)
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(loading_text, area);
            return Ok(());
        }

        let items: Vec<ListItem> = self
            .get_flat_items()
            .into_iter()
            .enumerate()
            .map(|(i, (text, _, _))| {
                let style = if i == self.selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_widget(list, area);
        Ok(())
    }
}
