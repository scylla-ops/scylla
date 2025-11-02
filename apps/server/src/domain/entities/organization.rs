use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{Description, OrganizationId, OrganizationName};
use chrono::{DateTime, Utc};

/// Organization domain entity
#[derive(Debug, Clone)]
pub struct Organization {
    id: OrganizationId,
    name: OrganizationName,
    description: Option<Description>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Organization {
    /// Create a new organization (for reconstruction from database)
    pub fn new(
        id: OrganizationId,
        name: OrganizationName,
        description: Option<Description>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        Ok(Self {
            id,
            name,
            description,
            is_active,
            created_at,
            updated_at,
        })
    }

    /// Create a new organization
    pub fn create(name: OrganizationName, description: Option<Description>) -> DomainResult<Self> {
        let now = Utc::now();
        Ok(Self {
            id: OrganizationId::generate(),
            name,
            description,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    /// Update the organization name
    pub fn update_name(&mut self, name: OrganizationName) -> DomainResult<()> {
        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update the organization description
    pub fn update_description(&mut self, description: Option<Description>) -> DomainResult<()> {
        self.description = description;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Toggle the organization active state
    pub fn toggle_active(&mut self) -> DomainResult<()> {
        self.is_active = !self.is_active;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Deactivate the organization
    pub fn deactivate(&mut self) -> DomainResult<()> {
        if !self.is_active {
            return Err(DomainError::business_rule(
                "Organization is already inactive",
            ));
        }
        self.is_active = false;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Activate the organization
    pub fn activate(&mut self) -> DomainResult<()> {
        if self.is_active {
            return Err(DomainError::business_rule("Organization is already active"));
        }
        self.is_active = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    // Getters
    pub fn id(&self) -> &OrganizationId {
        &self.id
    }

    pub fn name(&self) -> &OrganizationName {
        &self.name
    }

    pub fn description(&self) -> Option<&Description> {
        self.description.as_ref()
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

    /// Helper to create a test organization name
    fn test_org_name() -> OrganizationName {
        OrganizationName::new("Test Org".to_string()).unwrap()
    }

    /// Helper to create a different test organization name
    fn other_org_name() -> OrganizationName {
        OrganizationName::new("Other Org".to_string()).unwrap()
    }

    /// Helper to create a test description
    fn test_description() -> Description {
        Description::new("Test description".to_string()).unwrap()
    }

    // ===== Tests for Organization::create() =====

    #[test]
    fn test_create_organization_sets_default_values() {
        let name = test_org_name();
        let description = Some(test_description());

        let result = Organization::create(name, description);

        assert!(result.is_ok(), "Creating organization should succeed");
        let org = result.unwrap();

        assert_eq!(org.name().as_str(), "Test Org");
        assert!(org.description().is_some());
        assert!(
            org.is_active(),
            "New organization should be active by default"
        );

        // Verify timestamps are set
        assert_eq!(org.created_at(), org.updated_at());
    }

    #[test]
    fn test_create_organization_without_description() {
        let name = test_org_name();

        let result = Organization::create(name, None);

        assert!(result.is_ok());
        let org = result.unwrap();

        assert_eq!(org.name().as_str(), "Test Org");
        assert!(org.description().is_none());
        assert!(org.is_active());
    }

    #[test]
    fn test_create_organization_generates_unique_id() {
        let name1 = test_org_name();
        let name2 = test_org_name();

        let org1 = Organization::create(name1, None).unwrap();
        let org2 = Organization::create(name2, None).unwrap();

        assert_ne!(org1.id().as_str(), org2.id().as_str());
    }

    // ===== Tests for Organization::activate() =====

    #[test]
    fn test_activate_inactive_organization_succeeds() {
        let name = test_org_name();
        let mut org = Organization::create(name, None).unwrap();

        // First deactivate
        org.deactivate().unwrap();
        assert!(!org.is_active());

        let original_updated_at = org.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(1));

        // Now activate
        let result = org.activate();

        assert!(
            result.is_ok(),
            "Activating inactive organization should succeed"
        );
        assert!(
            org.is_active(),
            "Organization should be active after activation"
        );
        assert!(
            org.updated_at() > original_updated_at,
            "updated_at should be updated when organization is activated"
        );
    }

    #[test]
    fn test_activate_already_active_organization_fails() {
        let name = test_org_name();
        let mut org = Organization::create(name, None).unwrap();

        // Organization is already active by default
        assert!(org.is_active());

        let result = org.activate();

        assert!(
            result.is_err(),
            "Activating already active organization should fail"
        );

        if let Err(DomainError::BusinessRule(msg)) = result {
            assert_eq!(msg, "Organization is already active");
        } else {
            panic!("Expected BusinessRule error");
        }
    }

    // ===== Tests for Organization::deactivate() =====

    #[test]
    fn test_deactivate_active_organization_succeeds() {
        let name = test_org_name();
        let mut org = Organization::create(name, None).unwrap();

        assert!(org.is_active());
        let original_updated_at = org.updated_at();

        std::thread::sleep(std::time::Duration::from_millis(1));

        let result = org.deactivate();

        assert!(
            result.is_ok(),
            "Deactivating active organization should succeed"
        );
        assert!(
            !org.is_active(),
            "Organization should be inactive after deactivation"
        );
        assert!(
            org.updated_at() > original_updated_at,
            "updated_at should be updated when organization is deactivated"
        );
    }

    #[test]
    fn test_deactivate_already_inactive_organization_fails() {
        let name = test_org_name();
        let mut org = Organization::create(name, None).unwrap();

        // First deactivate
        org.deactivate().unwrap();
        assert!(!org.is_active());

        // Try to deactivate again
        let result = org.deactivate();

        assert!(
            result.is_err(),
            "Deactivating already inactive organization should fail"
        );

        if let Err(DomainError::BusinessRule(msg)) = result {
            assert_eq!(msg, "Organization is already inactive");
        } else {
            panic!("Expected BusinessRule error");
        }
    }

    // ===== Tests for Organization::toggle_active() =====

    #[test]
    fn test_toggle_active_from_active_to_inactive() {
        let name = test_org_name();
        let mut org = Organization::create(name, None).unwrap();

        assert!(org.is_active());

        let result = org.toggle_active();

        assert!(result.is_ok());
        assert!(
            !org.is_active(),
            "Organization should be inactive after toggle"
        );
    }

    #[test]
    fn test_toggle_active_from_inactive_to_active() {
        let name = test_org_name();
        let mut org = Organization::create(name, None).unwrap();

        // First deactivate
        org.deactivate().unwrap();
        assert!(!org.is_active());

        let result = org.toggle_active();

        assert!(result.is_ok());
        assert!(
            org.is_active(),
            "Organization should be active after toggle"
        );
    }

    // ===== Tests for Organization::update_name() =====

    #[test]
    fn test_update_name_succeeds() {
        let name = test_org_name();
        let mut org = Organization::create(name, None).unwrap();

        let original_updated_at = org.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(1));

        let new_name = other_org_name();
        let result = org.update_name(new_name);

        assert!(result.is_ok(), "Updating organization name should succeed");
        assert_eq!(org.name().as_str(), "Other Org");
        assert!(
            org.updated_at() > original_updated_at,
            "updated_at should be updated when name changes"
        );
    }

    // ===== Tests for Organization::update_description() =====

    #[test]
    fn test_update_description_add_description() {
        let name = test_org_name();
        let mut org = Organization::create(name, None).unwrap();

        assert!(org.description().is_none());

        let description = test_description();
        let result = org.update_description(Some(description));

        assert!(result.is_ok());
        assert!(org.description().is_some());
        assert_eq!(org.description().unwrap().as_str(), "Test description");
    }

    #[test]
    fn test_update_description_remove_description() {
        let name = test_org_name();
        let description = Some(test_description());
        let mut org = Organization::create(name, description).unwrap();

        assert!(org.description().is_some());

        let result = org.update_description(None);

        assert!(result.is_ok());
        assert!(org.description().is_none());
    }

    #[test]
    fn test_update_description_updates_timestamp() {
        let name = test_org_name();
        let mut org = Organization::create(name, None).unwrap();

        let original_updated_at = org.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(1));

        let result = org.update_description(Some(test_description()));

        assert!(result.is_ok());
        assert!(
            org.updated_at() > original_updated_at,
            "updated_at should be updated when description changes"
        );
    }

    // ===== Tests for Organization::new() (reconstruction from database) =====

    #[test]
    fn test_new_reconstructs_organization_from_database() {
        let id = OrganizationId::generate();
        let name = test_org_name();
        let description = Some(test_description());
        let is_active = false;
        let created_at = Utc::now();
        let updated_at = Utc::now();

        let result = Organization::new(
            id.clone(),
            name,
            description,
            is_active,
            created_at,
            updated_at,
        );

        assert!(
            result.is_ok(),
            "Reconstructing organization from database should succeed"
        );

        let org = result.unwrap();
        assert_eq!(org.id().as_str(), id.as_str());
        assert!(!org.is_active());
        assert_eq!(org.created_at(), created_at);
        assert_eq!(org.updated_at(), updated_at);
    }

    // ===== Integration tests for state transitions =====

    #[test]
    fn test_organization_lifecycle_activate_deactivate_cycle() {
        let name = test_org_name();
        let mut org = Organization::create(name, None).unwrap();

        // Organization starts active
        assert!(org.is_active());

        // Deactivate
        org.deactivate().expect("First deactivation should succeed");
        assert!(!org.is_active());

        // Can't deactivate again
        assert!(org.deactivate().is_err());

        // Activate
        org.activate().expect("Activation should succeed");
        assert!(org.is_active());

        // Can't activate again
        assert!(org.activate().is_err());
    }

    #[test]
    fn test_organization_lifecycle_with_toggle() {
        let name = test_org_name();
        let mut org = Organization::create(name, None).unwrap();

        // Organization starts active
        assert!(org.is_active());

        // Toggle to inactive
        org.toggle_active().expect("Toggle should succeed");
        assert!(!org.is_active());

        // Toggle to active
        org.toggle_active().expect("Toggle should succeed");
        assert!(org.is_active());

        // Toggle always succeeds regardless of current state
        org.toggle_active().expect("Toggle should succeed");
        assert!(!org.is_active());
    }
}
