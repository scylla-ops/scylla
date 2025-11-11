use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::{
    action::Action,
    components::{Component, fps::FpsCounter, login::Login, main_view::MainView},
    config::Config,
    profile::{ProfileStorage, UserProfile},
    tui::{Event, Tui},
};

use protocol::services::{
    auth::{LoginRequest, auth_service_client::AuthServiceClient},
    organization::{
        ListUserOrganizationsRequest, organization_service_client::OrganizationServiceClient,
    },
    project::{ListProjectsRequest, project_service_client::ProjectServiceClient},
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    auth_token: Option<String>,
    user_id: Option<String>,
    username: Option<String>,
    grpc_endpoint: String,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Login,
    Main,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(Login::new()), Box::new(FpsCounter::default())],
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            mode: Mode::Login,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            auth_token: None,
            user_id: None,
            username: None,
            grpc_endpoint: "http://127.0.0.1:50051".to_string(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            .mouse(true)
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }
        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();
        let Some(keymap) = self.config.keybindings.get(&self.mode) else {
            return Ok(());
        };
        match keymap.get(&vec![key]) {
            Some(action) => {
                info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
            }
            _ => {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                }
            }
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action.clone() {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                Action::Login { username, password } => {
                    self.username = Some(username.clone());
                    self.handle_login(username, password);
                }
                Action::LoginSuccess { token, user_id } => {
                    self.auth_token = Some(token);
                    self.user_id = Some(user_id);
                    self.mode = Mode::Main;

                    // Save the profile for future use
                    if let Err(e) = self.save_login_profile() {
                        debug!("Failed to save profile: {}", e);
                    }

                    // Switch to main view
                    self.components =
                        vec![Box::new(MainView::new()), Box::new(FpsCounter::default())];
                    // Re-register handlers for new components
                    for component in self.components.iter_mut() {
                        component.register_action_handler(self.action_tx.clone())?;
                        component.register_config_handler(self.config.clone())?;
                        component.init(tui.size()?)?;
                    }
                }
                Action::LoadOrganizations => {
                    self.handle_load_organizations();
                }
                Action::LoadProjects { organization_id } => {
                    self.handle_load_projects(organization_id);
                }
                _ => {}
            }
            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_login(&self, username: String, password: String) {
        let action_tx = self.action_tx.clone();
        let endpoint = self.grpc_endpoint.clone();
        tokio::spawn(async move {
            match AuthServiceClient::connect(endpoint).await {
                Ok(mut client) => {
                    let request = tonic::Request::new(LoginRequest { username, password });
                    match client.login(request).await {
                        Ok(response) => {
                            let response = response.into_inner();
                            let _ = action_tx.send(Action::LoginSuccess {
                                token: response.token,
                                user_id: response.user_id,
                            });
                        }
                        Err(e) => {
                            let _ = action_tx.send(Action::LoginFailed {
                                error: format!("Login failed: {}", e),
                            });
                        }
                    }
                }
                Err(e) => {
                    let _ = action_tx.send(Action::LoginFailed {
                        error: format!("Connection failed: {}", e),
                    });
                }
            }
        });
    }

    fn handle_load_organizations(&self) {
        let action_tx = self.action_tx.clone();
        let endpoint = self.grpc_endpoint.clone();
        let user_id = self.user_id.clone();
        let auth_token = self.auth_token.clone();

        tokio::spawn(async move {
            if let (Some(user_id), Some(token)) = (user_id, auth_token) {
                match OrganizationServiceClient::connect(endpoint).await {
                    Ok(mut client) => {
                        let mut request = tonic::Request::new(ListUserOrganizationsRequest {
                            user_id,
                            pagination: None,
                        });

                        // Add Authorization header with Bearer token
                        request.metadata_mut().insert(
                            "authorization",
                            format!("Bearer {}", token).parse().unwrap(),
                        );

                        match client.list_user_organizations(request).await {
                            Ok(response) => {
                                let organizations = response
                                    .into_inner()
                                    .organizations
                                    .into_iter()
                                    .map(|org| (org.organization_id, org.name))
                                    .collect();
                                let _ =
                                    action_tx.send(Action::OrganizationsLoaded { organizations });
                            }
                            Err(e) => {
                                let _ = action_tx.send(Action::Error(format!(
                                    "Failed to load organizations: {}",
                                    e
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::Error(format!("Connection failed: {}", e)));
                    }
                }
            }
        });
    }

    fn handle_load_projects(&self, organization_id: String) {
        let action_tx = self.action_tx.clone();
        let endpoint = self.grpc_endpoint.clone();
        let auth_token = self.auth_token.clone();

        tokio::spawn(async move {
            if let Some(token) = auth_token {
                match ProjectServiceClient::connect(endpoint).await {
                    Ok(mut client) => {
                        let mut request =
                            tonic::Request::new(ListProjectsRequest { pagination: None });

                        // Add Authorization header with Bearer token
                        request.metadata_mut().insert(
                            "authorization",
                            format!("Bearer {}", token).parse().unwrap(),
                        );

                        match client.list_projects(request).await {
                            Ok(response) => {
                                let projects = response
                                    .into_inner()
                                    .projects
                                    .into_iter()
                                    .filter(|p| p.organization_id == organization_id)
                                    .map(|p| (p.project_id, p.name))
                                    .collect();
                                let _ = action_tx.send(Action::ProjectsLoaded {
                                    organization_id,
                                    projects,
                                });
                            }
                            Err(e) => {
                                let _ = action_tx
                                    .send(Action::Error(format!("Failed to load projects: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::Error(format!("Connection failed: {}", e)));
                    }
                }
            }
        });
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, frame.area()) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {:?}", err)));
                }
            }
        })?;
        Ok(())
    }

    fn save_login_profile(&self) -> Result<()> {
        if let Some(username) = &self.username {
            let mut storage = ProfileStorage::load().unwrap_or_default();
            storage.add_profile(UserProfile::new(username.clone()));
            storage.save()?;
        }
        Ok(())
    }
}
