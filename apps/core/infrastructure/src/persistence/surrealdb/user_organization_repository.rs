use async_trait::async_trait;
use domain::entities::{OrganizationId, UserId, UserOrganization, UserOrganizationId};
use domain::errors::{DomainError, DomainResult};
use domain::ports::UserOrganizationRepository;
use domain::value_objects::user_organization::user_organization_role::UserOrganizationRole;
use domain::value_objects::{PaginatedResult, PaginationParams};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::RecordId;
use surrealdb_types::SurrealValue;

pub struct SurrealUserOrganizationRepository {
    db: Surreal<Any>,
}

impl SurrealUserOrganizationRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserOrganizationRepository for SurrealUserOrganizationRepository {
    async fn create(&self, user_organization: &UserOrganization) -> DomainResult<UserOrganization> {
        let db = self.db.clone();
        let user_organization = user_organization.clone();
        let created: Option<UserOrganization> = db
            .create(RecordId::new(
                UserOrganizationId::table_name(),
                user_organization.id().as_str(),
            ))
            .content(user_organization.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        created.ok_or_else(|| DomainError::infrastructure("Failed to create user organization"))
    }

    async fn find_by_id(&self, id: &UserOrganizationId) -> DomainResult<UserOrganization> {
        let db = self.db.clone();
        let id = id.clone();
        let result: Option<UserOrganization> = db
            .select(RecordId::new(UserOrganizationId::table_name(), id.as_str()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        result.ok_or_else(|| DomainError::not_found("UserOrganization", id.to_string()))
    }

    async fn find_by_user_and_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> DomainResult<UserOrganization> {
        let db = self.db.clone();
        let user_record = user_id.clone().into_value();
        let organization_record = organization_id.clone().into_value();
        let table = UserOrganizationId::table_name().to_string();
        let mut results: Vec<UserOrganization> = db
                .query("SELECT * FROM type::table($table) WHERE user_id = $user_id AND organization_id = $organization_id LIMIT 1")
                .bind(("table", table))
                .bind(("user_id", user_record))
                .bind(("organization_id", organization_record))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        results.pop().ok_or_else(|| {
            DomainError::not_found(
                "UserOrganization",
                format!("user_id={}, organization_id={}", user_id, organization_id),
            )
        })
    }

    async fn update(&self, user_organization: &UserOrganization) -> DomainResult<UserOrganization> {
        let db = self.db.clone();
        let user_organization = user_organization.clone();
        let updated: Option<UserOrganization> = db
            .update(RecordId::new(
                UserOrganizationId::table_name(),
                user_organization.id().as_str(),
            ))
            .content(user_organization.clone())
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        updated.ok_or_else(|| {
            DomainError::not_found("UserOrganization", user_organization.id().to_string())
        })
    }

    async fn delete(&self, id: &UserOrganizationId) -> DomainResult<()> {
        let db = self.db.clone();
        let id = id.clone();
        db.delete::<Option<UserOrganization>>(RecordId::new(
            UserOrganizationId::table_name(),
            id.as_str(),
        ))
        .await
        .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_all(&self) -> DomainResult<Vec<UserOrganization>> {
        let db = self.db.clone();
        let table = UserOrganizationId::table_name().to_string();
        let results: Vec<UserOrganization> = db
            .query("SELECT * FROM type::table($table)")
            .bind(("table", table))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take(0)
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        Ok(results)
    }

    async fn list_organizations_for_user(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<OrganizationId>> {
        let db = self.db.clone();
        let user_record = user_id.clone().into_value();
        let params = pagination.cloned().unwrap_or_default();
        let table = UserOrganizationId::table_name().to_string();
        let count_result: Vec<i64> = db
            .query("SELECT count() FROM type::table($table) WHERE user_id = $user_id GROUP ALL")
            .bind(("table", table.clone()))
            .bind(("user_id", user_record.clone()))
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
            .take("count")
            .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

        let results: Vec<UserOrganization> = db
                .query("SELECT * FROM type::table($table) WHERE user_id = $user_id ORDER BY joined_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("user_id", user_record))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let org_ids: Vec<OrganizationId> = results
            .into_iter()
            .map(|uo| uo.organization_id().clone())
            .collect();

        Ok(PaginatedResult::new(org_ids, &params, total_count))
    }

    async fn list_users_in_organization(
        &self,
        organization_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        let db = self.db.clone();
        let organization_record = organization_id.clone().into_value();
        let params = pagination.cloned().unwrap_or_default();
        let table = UserOrganizationId::table_name().to_string();
        let count_result: Vec<i64> = db
                .query("SELECT count() FROM type::table($table) WHERE organization_id = $organization_id GROUP ALL")
                .bind(("table", table.clone()))
                .bind(("organization_id", organization_record.clone()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take("count")
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let total_count = count_result.first().copied().unwrap_or(0) as u64;

        let results: Vec<UserOrganization> = db
                .query("SELECT * FROM type::table($table) WHERE organization_id = $organization_id ORDER BY joined_at DESC LIMIT $limit START $start")
                .bind(("table", table))
                .bind(("organization_id", organization_record))
                .bind(("limit", params.limit()))
                .bind(("start", params.offset()))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?
                .take(0)
                .map_err(|e| DomainError::infrastructure(format!("Query error: {}", e)))?;

        let user_ids: Vec<UserId> = results.into_iter().map(|uo| uo.user_id().clone()).collect();

        Ok(PaginatedResult::new(user_ids, &params, total_count))
    }

    async fn add_user_to_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
        role: &str,
    ) -> DomainResult<UserOrganizationId> {
        let db = self.db.clone();
        let user_id = user_id.clone();
        let organization_id = organization_id.clone();
        let role = role.to_string();
        let parsed_role = UserOrganizationRole::new(&role)?;

        let user_org = UserOrganization::create(user_id, organization_id, parsed_role)?;
        let id = user_org.id().clone();

        let _created: Option<UserOrganization> = db
            .create(RecordId::new(
                UserOrganizationId::table_name(),
                user_org.id().as_str(),
            ))
            .content(user_org)
            .await
            .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(id)
    }

    async fn remove_user_from_organization(
        &self,
        user_id: &UserId,
        organization_id: &OrganizationId,
    ) -> DomainResult<()> {
        let db = self.db.clone();
        let user_record = user_id.clone().into_value();
        let organization_record = organization_id.clone().into_value();
        let table = UserOrganizationId::table_name().to_string();
        db.query("DELETE FROM type::table($table) WHERE user_id = $user_id AND organization_id = $organization_id")
                .bind(("table", table))
                .bind(("user_id", user_record))
                .bind(("organization_id", organization_record))
                .await
                .map_err(|e| DomainError::infrastructure(format!("Database error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::init_db;
    use domain::entities::{OrganizationId, UserId, UserOrganization};
    use domain::value_objects::PaginationParams;
    use domain::value_objects::user_organization::user_organization_role::UserOrganizationRole;

    async fn setup() -> Surreal<Any> {
        init_db(&[UserOrganizationId::table_name()]).await
    }

    fn test_user_id() -> UserId {
        UserId::generate()
    }

    fn test_org_id() -> OrganizationId {
        OrganizationId::generate()
    }

    #[tokio::test]
    async fn test_create() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let user_id = test_user_id();
        let org_id = test_org_id();
        let role = UserOrganizationRole::member();
        let user_org = UserOrganization::create(user_id.clone(), org_id.clone(), role).unwrap();
        let user_org_id = user_org.id().clone();

        let created = repo
            .create(&user_org)
            .await
            .expect("Failed to create user organization");
        assert_eq!(created.id(), &user_org_id);
        assert_eq!(created.user_id(), &user_id);
        assert_eq!(created.organization_id(), &org_id);
        assert_eq!(created.role().as_str(), "member");
        assert_eq!(created.joined_at(), user_org.joined_at());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let user_id = test_user_id();
        let org_id = test_org_id();
        let user_org = UserOrganization::create(
            user_id.clone(),
            org_id.clone(),
            UserOrganizationRole::owner(),
        )
        .unwrap();
        let user_org_id = user_org.id().clone();

        repo.create(&user_org).await.expect("Failed to create");

        let found = repo
            .find_by_id(&user_org_id)
            .await
            .expect("Failed to find user organization by id");
        assert_eq!(found.id(), &user_org_id);
        assert_eq!(found.user_id(), &user_id);
        assert_eq!(found.organization_id(), &org_id);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let fake_id = UserOrganizationId::generate();
        let result = repo.find_by_id(&fake_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_by_user_and_organization() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let user_id = test_user_id();
        let org_id = test_org_id();
        let user_org = UserOrganization::create(
            user_id.clone(),
            org_id.clone(),
            UserOrganizationRole::admin(),
        )
        .unwrap();

        repo.create(&user_org).await.expect("Failed to create");

        let found = repo
            .find_by_user_and_organization(&user_id, &org_id)
            .await
            .expect("Failed to find by user and organization");
        assert_eq!(found.user_id(), &user_id);
        assert_eq!(found.organization_id(), &org_id);
        assert_eq!(found.role().as_str(), "admin");
    }

    #[tokio::test]
    async fn test_find_by_user_and_organization_not_found() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let result = repo
            .find_by_user_and_organization(&test_user_id(), &test_org_id())
            .await;
        assert!(result.is_err());
    }

    // #[tokio::test]
    // async fn test_update() {
    //     let db = setup().await;
    //     let repo = SurrealUserOrganizationRepository::new(db);

    //     let user_id = test_user_id();
    //     let org_id = test_org_id();
    //     let user_org = UserOrganization::create(
    //         user_id.clone(),
    //         org_id.clone(),
    //         UserOrganizationRole::member(),
    //     )
    //     .unwrap();
    //     let user_org_id = user_org.id().clone();

    //     let orga = repo.create(&user_org).await.expect("Failed to create");

    //     // Recreate with a different role to simulate an update
    //     let updated_user_org = UserOrganization::create(
    //         orga.user_id().clone(),
    //         orga.organization_id().clone(),
    //         UserOrganizationRole::admin(),
    //     )
    //     .unwrap();
    //     // We need to update the existing record, so we use the original entity with new content
    //     // The update method replaces the full content by ID, so we just pass the original with changed data
    //     let updated = repo
    //         .update(&updated_user_org)
    //         .await
    //         .expect("Failed to update");

    //     assert_eq!(updated.id(), orga.user_id());
    //     assert_eq!(updated.user_id(), &user_id);
    //     assert_eq!(updated.organization_id(), &org_id);
    // }

    #[tokio::test]
    async fn test_delete() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let user_id = test_user_id();
        let org_id = test_org_id();
        let user_org =
            UserOrganization::create(user_id, org_id, UserOrganizationRole::member()).unwrap();
        let user_org_id = user_org.id().clone();

        repo.create(&user_org).await.expect("Failed to create");

        repo.delete(&user_org_id).await.expect("Failed to delete");

        let result = repo.find_by_id(&user_org_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_all() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let uo1 = UserOrganization::create(
            test_user_id(),
            test_org_id(),
            UserOrganizationRole::member(),
        )
        .unwrap();
        repo.create(&uo1).await.expect("Failed to create uo1");

        let uo2 =
            UserOrganization::create(test_user_id(), test_org_id(), UserOrganizationRole::owner())
                .unwrap();
        repo.create(&uo2).await.expect("Failed to create uo2");

        let result = repo.list_all().await.expect("Failed to list all");
        assert!(result.len() >= 2);
    }

    #[tokio::test]
    async fn test_list_organizations_for_user() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let user_id = test_user_id();
        let org1 = test_org_id();
        let org2 = test_org_id();
        let other_user = test_user_id();

        let uo1 = UserOrganization::create(
            user_id.clone(),
            org1.clone(),
            UserOrganizationRole::member(),
        )
        .unwrap();
        repo.create(&uo1).await.expect("Failed to create uo1");

        let uo2 =
            UserOrganization::create(user_id.clone(), org2.clone(), UserOrganizationRole::admin())
                .unwrap();
        repo.create(&uo2).await.expect("Failed to create uo2");

        // Another user's membership — should not appear
        let uo3 =
            UserOrganization::create(other_user, test_org_id(), UserOrganizationRole::owner())
                .unwrap();
        repo.create(&uo3).await.expect("Failed to create uo3");

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_organizations_for_user(&user_id, Some(&pagination))
            .await
            .expect("Failed to list organizations for user");
        assert_eq!(result.items().len(), 2);
        assert_eq!(result.metadata().total_count(), 2);
    }

    #[tokio::test]
    async fn test_list_users_in_organization() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let org_id = test_org_id();
        let user1 = test_user_id();
        let user2 = test_user_id();
        let other_org = test_org_id();

        let uo1 = UserOrganization::create(
            user1.clone(),
            org_id.clone(),
            UserOrganizationRole::member(),
        )
        .unwrap();
        repo.create(&uo1).await.expect("Failed to create uo1");

        let uo2 =
            UserOrganization::create(user2.clone(), org_id.clone(), UserOrganizationRole::admin())
                .unwrap();
        repo.create(&uo2).await.expect("Failed to create uo2");

        // Different org — should not appear
        let uo3 =
            UserOrganization::create(test_user_id(), other_org, UserOrganizationRole::owner())
                .unwrap();
        repo.create(&uo3).await.expect("Failed to create uo3");

        let pagination = PaginationParams::new(1, 20).unwrap();
        let result = repo
            .list_users_in_organization(&org_id, Some(&pagination))
            .await
            .expect("Failed to list users in organization");
        assert_eq!(result.items().len(), 2);
        assert_eq!(result.metadata().total_count(), 2);
    }

    #[tokio::test]
    async fn test_add_user_to_organization() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let user_id = test_user_id();
        let org_id = test_org_id();

        let id = repo
            .add_user_to_organization(&user_id, &org_id, "owner")
            .await
            .expect("Failed to add user to organization");

        let found = repo
            .find_by_id(&id)
            .await
            .expect("Failed to find added user organization");
        assert_eq!(found.user_id(), &user_id);
        assert_eq!(found.organization_id(), &org_id);
        assert_eq!(found.role().as_str(), "owner");
    }

    #[tokio::test]
    async fn test_add_user_to_organization_invalid_role() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let result = repo
            .add_user_to_organization(&test_user_id(), &test_org_id(), "invalid_role")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_user_from_organization() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        let user_id = test_user_id();
        let org_id = test_org_id();
        let user_org = UserOrganization::create(
            user_id.clone(),
            org_id.clone(),
            UserOrganizationRole::member(),
        )
        .unwrap();
        let user_org_id = user_org.id().clone();

        repo.create(&user_org).await.expect("Failed to create");

        repo.remove_user_from_organization(&user_id, &org_id)
            .await
            .expect("Failed to remove user from organization");

        let result = repo.find_by_id(&user_org_id).await;

        let result_by_pair = repo.find_by_user_and_organization(&user_id, &org_id).await;
        assert!(result.is_err() || result_by_pair.is_err());
    }

    #[tokio::test]
    async fn test_remove_user_from_organization_nonexistent() {
        let db = setup().await;
        let repo = SurrealUserOrganizationRepository::new(db);

        // Removing a non-existent association should not error
        let result = repo
            .remove_user_from_organization(&test_user_id(), &test_org_id())
            .await;
        assert!(result.is_ok());
    }
}
