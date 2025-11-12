use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    // Authentication actions
    Login {
        username: String,
        password: String,
    },
    LoginSuccess {
        token: String,
        user_id: String,
    },
    LoginFailed {
        error: String,
    },
    // Navigation actions
    FocusUsername,
    FocusPassword,
    SubmitLogin,
    // Sidebar actions
    LoadOrganizations,
    OrganizationsLoaded {
        organizations: Vec<(String, String)>,
    }, // (id, name)
    LoadProjects {
        organization_id: String,
    },
    ProjectsLoaded {
        organization_id: String,
        projects: Vec<(String, String)>,
    }, // (id, name)
    ToggleOrganization {
        organization_id: String,
    },
    SelectOrganization {
        organization_id: String,
        organization_name: String,
    },
    SelectProject {
        project_id: String,
        project_name: String,
    },
}
