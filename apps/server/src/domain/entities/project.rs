use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{Description, OrganizationId, ProjectId, ProjectName};
use chrono::{DateTime, Utc};

/// Project domain entity
#[derive(Debug, Clone)]
pub struct Project {
    id: ProjectId,
    name: ProjectName,
    description: Option<Description>,
    organization_id: OrganizationId,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Project {
    /// Create a new project (for reconstruction from database)
    pub fn new(
        id: ProjectId,
        name: ProjectName,
        description: Option<Description>,
        organization_id: OrganizationId,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        Ok(Self {
            id,
            name,
            description,
            organization_id,
            is_active,
            created_at,
            updated_at,
        })
    }

    /// Create a new project
    pub fn create(
        name: ProjectName,
        description: Option<Description>,
        organization_id: OrganizationId,
    ) -> DomainResult<Self> {
        let now = Utc::now();
        Ok(Self {
            id: ProjectId::generate(),
            name,
            description,
            organization_id,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update the project name
    pub fn update_name(&mut self, name: ProjectName) -> DomainResult<()> {
        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update the project description
    pub fn update_description(&mut self, description: Option<Description>) -> DomainResult<()> {
        self.description = description;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Toggle the project active state
    pub fn toggle_active(&mut self) -> DomainResult<()> {
        self.is_active = !self.is_active;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Deactivate the project
    pub fn deactivate(&mut self) -> DomainResult<()> {
        if !self.is_active {
            return Err(DomainError::business_rule("Project is already inactive"));
        }
        self.is_active = false;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Activate the project
    pub fn activate(&mut self) -> DomainResult<()> {
        if self.is_active {
            return Err(DomainError::business_rule("Project is already active"));
        }
        self.is_active = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    // Getters
    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    pub fn name(&self) -> &ProjectName {
        &self.name
    }

    pub fn description(&self) -> Option<&Description> {
        self.description.as_ref()
    }

    pub fn organization_id(&self) -> &OrganizationId {
        &self.organization_id
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test project name
    fn test_project_name() -> ProjectName {
        ProjectName::new("Test Project".to_string()).unwrap()
    }

    /// Helper to create a different test project name
    fn other_project_name() -> ProjectName {
        ProjectName::new("Other Project".to_string()).unwrap()
    }

    /// Helper to create a test description
    fn test_description() -> Description {
        Description::new("Test project description".to_string()).unwrap()
    }

    /// Helper to create a test organization ID
    fn test_organization_id() -> OrganizationId {
        OrganizationId::generate()
    }

    // ===== Tests for Project::create() =====

    #[test]
    fn test_create_project_sets_default_values() {
        let name = test_project_name();
        let description = Some(test_description());
        let org_id = test_organization_id();

        let result = Project::create(name, description, org_id.clone());

        assert!(result.is_ok(), "Creating project should succeed");
        let project = result.unwrap();

        assert_eq!(project.name().as_str(), "Test Project");
        assert!(project.description().is_some());
        assert_eq!(project.organization_id().as_str(), org_id.as_str());
        assert!(
            project.is_active(),
            "New project should be active by default"
        );

        // Verify timestamps are set
        assert_eq!(project.created_at(), project.updated_at());
    }

    #[test]
    fn test_create_project_without_description() {
        let name = test_project_name();
        let org_id = test_organization_id();

        let result = Project::create(name, None, org_id);

        assert!(result.is_ok());
        let project = result.unwrap();

        assert_eq!(project.name().as_str(), "Test Project");
        assert!(project.description().is_none());
        assert!(project.is_active());
    }

    #[test]
    fn test_create_project_generates_unique_id() {
        let name1 = test_project_name();
        let name2 = test_project_name();
        let org_id = test_organization_id();

        let project1 = Project::create(name1, None, org_id.clone()).unwrap();
        let project2 = Project::create(name2, None, org_id).unwrap();

        assert_ne!(project1.id().as_str(), project2.id().as_str());
    }

    // ===== Tests for Project::activate() =====

    #[test]
    fn test_activate_inactive_project_succeeds() {
        let name = test_project_name();
        let org_id = test_organization_id();
        let mut project = Project::create(name, None, org_id).unwrap();

        // First deactivate
        project.deactivate().unwrap();
        assert!(!project.is_active());

        let original_updated_at = project.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(1));

        // Now activate
        let result = project.activate();

        assert!(result.is_ok(), "Activating inactive project should succeed");
        assert!(
            project.is_active(),
            "Project should be active after activation"
        );
        assert!(
            project.updated_at() > original_updated_at,
            "updated_at should be updated when project is activated"
        );
    }

    #[test]
    fn test_activate_already_active_project_fails() {
        let name = test_project_name();
        let org_id = test_organization_id();
        let mut project = Project::create(name, None, org_id).unwrap();

        // Project is already active by default
        assert!(project.is_active());

        let result = project.activate();

        assert!(
            result.is_err(),
            "Activating already active project should fail"
        );

        if let Err(DomainError::BusinessRule(msg)) = result {
            assert_eq!(msg, "Project is already active");
        } else {
            panic!("Expected BusinessRule error");
        }
    }

    // ===== Tests for Project::deactivate() =====

    #[test]
    fn test_deactivate_active_project_succeeds() {
        let name = test_project_name();
        let org_id = test_organization_id();
        let mut project = Project::create(name, None, org_id).unwrap();

        assert!(project.is_active());
        let original_updated_at = project.updated_at();

        std::thread::sleep(std::time::Duration::from_millis(1));

        let result = project.deactivate();

        assert!(result.is_ok(), "Deactivating active project should succeed");
        assert!(
            !project.is_active(),
            "Project should be inactive after deactivation"
        );
        assert!(
            project.updated_at() > original_updated_at,
            "updated_at should be updated when project is deactivated"
        );
    }

    #[test]
    fn test_deactivate_already_inactive_project_fails() {
        let name = test_project_name();
        let org_id = test_organization_id();
        let mut project = Project::create(name, None, org_id).unwrap();

        // First deactivate
        project.deactivate().unwrap();
        assert!(!project.is_active());

        // Try to deactivate again
        let result = project.deactivate();

        assert!(
            result.is_err(),
            "Deactivating already inactive project should fail"
        );

        if let Err(DomainError::BusinessRule(msg)) = result {
            assert_eq!(msg, "Project is already inactive");
        } else {
            panic!("Expected BusinessRule error");
        }
    }

    // ===== Tests for Project::toggle_active() =====

    #[test]
    fn test_toggle_active_from_active_to_inactive() {
        let name = test_project_name();
        let org_id = test_organization_id();
        let mut project = Project::create(name, None, org_id).unwrap();

        assert!(project.is_active());

        let result = project.toggle_active();

        assert!(result.is_ok());
        assert!(
            !project.is_active(),
            "Project should be inactive after toggle"
        );
    }

    #[test]
    fn test_toggle_active_from_inactive_to_active() {
        let name = test_project_name();
        let org_id = test_organization_id();
        let mut project = Project::create(name, None, org_id).unwrap();

        // First deactivate
        project.deactivate().unwrap();
        assert!(!project.is_active());

        let result = project.toggle_active();

        assert!(result.is_ok());
        assert!(project.is_active(), "Project should be active after toggle");
    }

    // ===== Tests for Project::update_name() =====

    #[test]
    fn test_update_name_succeeds() {
        let name = test_project_name();
        let org_id = test_organization_id();
        let mut project = Project::create(name, None, org_id).unwrap();

        let original_updated_at = project.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(1));

        let new_name = other_project_name();
        let result = project.update_name(new_name);

        assert!(result.is_ok(), "Updating project name should succeed");
        assert_eq!(project.name().as_str(), "Other Project");
        assert!(
            project.updated_at() > original_updated_at,
            "updated_at should be updated when name changes"
        );
    }

    // ===== Tests for Project::update_description() =====

    #[test]
    fn test_update_description_add_description() {
        let name = test_project_name();
        let org_id = test_organization_id();
        let mut project = Project::create(name, None, org_id).unwrap();

        assert!(project.description().is_none());

        let description = test_description();
        let result = project.update_description(Some(description));

        assert!(result.is_ok());
        assert!(project.description().is_some());
        assert_eq!(
            project.description().unwrap().as_str(),
            "Test project description"
        );
    }

    #[test]
    fn test_update_description_remove_description() {
        let name = test_project_name();
        let description = Some(test_description());
        let org_id = test_organization_id();
        let mut project = Project::create(name, description, org_id).unwrap();

        assert!(project.description().is_some());

        let result = project.update_description(None);

        assert!(result.is_ok());
        assert!(project.description().is_none());
    }

    #[test]
    fn test_update_description_updates_timestamp() {
        let name = test_project_name();
        let org_id = test_organization_id();
        let mut project = Project::create(name, None, org_id).unwrap();

        let original_updated_at = project.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(1));

        let result = project.update_description(Some(test_description()));

        assert!(result.is_ok());
        assert!(
            project.updated_at() > original_updated_at,
            "updated_at should be updated when description changes"
        );
    }

    // ===== Tests for Project::new() (reconstruction from database) =====

    #[test]
    fn test_new_reconstructs_project_from_database() {
        let id = ProjectId::generate();
        let name = test_project_name();
        let description = Some(test_description());
        let org_id = test_organization_id();
        let is_active = false;
        let created_at = Utc::now();
        let updated_at = Utc::now();

        let result = Project::new(
            id.clone(),
            name,
            description,
            org_id.clone(),
            is_active,
            created_at,
            updated_at,
        );

        assert!(
            result.is_ok(),
            "Reconstructing project from database should succeed"
        );

        let project = result.unwrap();
        assert_eq!(project.id().as_str(), id.as_str());
        assert_eq!(project.organization_id().as_str(), org_id.as_str());
        assert!(!project.is_active());
        assert_eq!(project.created_at(), created_at);
        assert_eq!(project.updated_at(), updated_at);
    }

    // ===== Integration tests for state transitions =====

    #[test]
    fn test_project_lifecycle_activate_deactivate_cycle() {
        let name = test_project_name();
        let org_id = test_organization_id();
        let mut project = Project::create(name, None, org_id).unwrap();

        // Project starts active
        assert!(project.is_active());

        // Deactivate
        project
            .deactivate()
            .expect("First deactivation should succeed");
        assert!(!project.is_active());

        // Can't deactivate again
        assert!(project.deactivate().is_err());

        // Activate
        project.activate().expect("Activation should succeed");
        assert!(project.is_active());

        // Can't activate again
        assert!(project.activate().is_err());
    }

    #[test]
    fn test_project_lifecycle_with_toggle() {
        let name = test_project_name();
        let org_id = test_organization_id();
        let mut project = Project::create(name, None, org_id).unwrap();

        // Project starts active
        assert!(project.is_active());

        // Toggle to inactive
        project.toggle_active().expect("Toggle should succeed");
        assert!(!project.is_active());

        // Toggle to active
        project.toggle_active().expect("Toggle should succeed");
        assert!(project.is_active());

        // Toggle always succeeds regardless of current state
        project.toggle_active().expect("Toggle should succeed");
        assert!(!project.is_active());
    }
}
