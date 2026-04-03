use crate::application::ports::{
    OrganizationRepository, UserOrganizationRepository, UserRepository,
};
use crate::domain::entities::{Organization, OrganizationId, User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::organization::{OrganizationDescription, OrganizationName};
use crate::domain::value_objects::{PaginatedResult, PaginationMetadata, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct OrganizationUseCases<
    O: OrganizationRepository,
    UO: UserOrganizationRepository,
    U: UserRepository,
> {
    org_repo: Arc<O>,
    user_org_repo: Arc<UO>,
    user_repo: Arc<U>,
}

impl<O: OrganizationRepository, UO: UserOrganizationRepository, U: UserRepository>
    OrganizationUseCases<O, UO, U>
{
    #[instrument(skip(self), fields(name = %name))]
    pub async fn create(
        &self,
        name: OrganizationName,
        description: Option<OrganizationDescription>,
    ) -> DomainResult<Organization> {
        if self.org_repo.name_exists(&name).await? {
            return Err(DomainError::conflict("Organization name already exists"));
        }

        let org = Organization::create(name, description)?;
        self.org_repo.create(&org).await
    }

    #[instrument(skip(self), fields(org_id = %id))]
    pub async fn get(&self, id: &OrganizationId) -> DomainResult<Organization> {
        self.org_repo.find_by_id(id).await
    }

    #[instrument(skip(self), fields(org_id = %id))]
    pub async fn update(
        &self,
        id: &OrganizationId,
        name: Option<OrganizationName>,
        description: Option<Option<OrganizationDescription>>,
    ) -> DomainResult<Organization> {
        let mut org = self.org_repo.find_by_id(id).await?;

        if let Some(new_name) = name {
            if self.org_repo.name_exists(&new_name).await? && org.name() != &new_name {
                return Err(DomainError::conflict("Organization name already exists"));
            }
            org.update_name(new_name)?;
        }
        if let Some(new_desc) = description {
            org.update_description(new_desc)?;
        }

        self.org_repo.update(&org).await
    }

    #[instrument(skip(self), fields(org_id = %id))]
    pub async fn toggle_active(&self, id: &OrganizationId) -> DomainResult<()> {
        let mut org = self.org_repo.find_by_id(id).await?;
        org.toggle_active()?;
        self.org_repo.update(&org).await?;
        Ok(())
    }

    #[instrument(skip(self), fields(org_id = %id))]
    pub async fn delete(&self, id: &OrganizationId) -> DomainResult<()> {
        self.org_repo.find_by_id(id).await?;
        self.org_repo.delete(id).await
    }

    #[instrument(skip(self))]
    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        self.org_repo.list_all(pagination).await
    }

    #[instrument(skip(self), fields(org_id = %org_id))]
    pub async fn list_users(
        &self,
        org_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<User>, PaginationMetadata)> {
        let paginated = self.user_org_repo.list_members(org_id, pagination).await?;
        let (user_ids, metadata) = paginated.into_parts();

        let mut users = Vec::with_capacity(user_ids.len());
        for user_id in &user_ids {
            let user = self.user_repo.find_by_id(user_id).await?;
            users.push(user);
        }

        Ok((users, metadata))
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn list_user_orgs(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<Organization>, PaginationMetadata)> {
        let paginated = self
            .user_org_repo
            .list_user_organizations(user_id, pagination)
            .await?;
        let (org_ids, metadata) = paginated.into_parts();

        let mut orgs = Vec::with_capacity(org_ids.len());
        for org_id in &org_ids {
            let org = self.org_repo.find_by_id(org_id).await?;
            orgs.push(org);
        }

        Ok((orgs, metadata))
    }

    #[instrument(skip(self), fields(user_id = %user_id, org_id = %org_id))]
    pub async fn add_user(&self, user_id: &UserId, org_id: &OrganizationId) -> DomainResult<()> {
        if self.user_org_repo.is_member(user_id, org_id).await? {
            return Err(DomainError::conflict(
                "User is already a member of this organization",
            ));
        }

        self.user_org_repo.add_member(user_id, org_id).await
    }

    #[instrument(skip(self), fields(user_id = %user_id, org_id = %org_id))]
    pub async fn remove_user(&self, user_id: &UserId, org_id: &OrganizationId) -> DomainResult<()> {
        self.user_org_repo.remove_member(user_id, org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{OrganizationRepository, UserOrganizationRepository, UserRepository};
    use crate::domain::value_objects::user::{PasswordHash, Username};
    use async_trait::async_trait;
    use std::sync::Arc;

    // ── Stubs ─────────────────────────────────────────────────────

    #[derive(Default)]
    struct StubOrgRepo {
        create_fn: Option<Box<dyn Fn(&Organization) -> DomainResult<Organization> + Send + Sync>>,
        find_by_id_fn: Option<Box<dyn Fn(&OrganizationId) -> DomainResult<Organization> + Send + Sync>>,
        update_fn: Option<Box<dyn Fn(&Organization) -> DomainResult<Organization> + Send + Sync>>,
        delete_fn: Option<Box<dyn Fn(&OrganizationId) -> DomainResult<()> + Send + Sync>>,
        name_exists_fn: Option<Box<dyn Fn(&OrganizationName) -> DomainResult<bool> + Send + Sync>>,
        list_all_fn: Option<Box<dyn Fn() -> DomainResult<PaginatedResult<Organization>> + Send + Sync>>,
    }

    #[async_trait]
    impl OrganizationRepository for StubOrgRepo {
        async fn create(&self, org: &Organization) -> DomainResult<Organization> {
            (self.create_fn.as_ref().unwrap())(org)
        }
        async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization> {
            (self.find_by_id_fn.as_ref().unwrap())(id)
        }
        async fn find_by_name(&self, _name: &OrganizationName) -> DomainResult<Organization> {
            unimplemented!()
        }
        async fn update(&self, org: &Organization) -> DomainResult<Organization> {
            (self.update_fn.as_ref().unwrap())(org)
        }
        async fn delete(&self, id: &OrganizationId) -> DomainResult<()> {
            (self.delete_fn.as_ref().unwrap())(id)
        }
        async fn list_all(&self, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Organization>> {
            (self.list_all_fn.as_ref().unwrap())()
        }
        async fn list_active(&self, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<Organization>> {
            unimplemented!()
        }
        async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool> {
            (self.name_exists_fn.as_ref().unwrap())(name)
        }
    }

    #[derive(Default)]
    struct StubUserOrgRepo {
        add_member_fn: Option<Box<dyn Fn(&UserId, &OrganizationId) -> DomainResult<()> + Send + Sync>>,
        remove_member_fn: Option<Box<dyn Fn(&UserId, &OrganizationId) -> DomainResult<()> + Send + Sync>>,
        is_member_fn: Option<Box<dyn Fn(&UserId, &OrganizationId) -> DomainResult<bool> + Send + Sync>>,
        list_members_fn: Option<Box<dyn Fn(&OrganizationId) -> DomainResult<PaginatedResult<UserId>> + Send + Sync>>,
    }

    #[async_trait]
    impl UserOrganizationRepository for StubUserOrgRepo {
        async fn add_member(&self, uid: &UserId, oid: &OrganizationId) -> DomainResult<()> {
            (self.add_member_fn.as_ref().unwrap())(uid, oid)
        }
        async fn remove_member(&self, uid: &UserId, oid: &OrganizationId) -> DomainResult<()> {
            (self.remove_member_fn.as_ref().unwrap())(uid, oid)
        }
        async fn is_member(&self, uid: &UserId, oid: &OrganizationId) -> DomainResult<bool> {
            (self.is_member_fn.as_ref().unwrap())(uid, oid)
        }
        async fn list_members(&self, oid: &OrganizationId, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<UserId>> {
            (self.list_members_fn.as_ref().unwrap())(oid)
        }
        async fn list_user_organizations(&self, _uid: &UserId, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<OrganizationId>> {
            unimplemented!()
        }
    }

    #[derive(Default)]
    struct StubUserRepo {
        find_by_id_fn: Option<Box<dyn Fn(&UserId) -> DomainResult<User> + Send + Sync>>,
    }

    #[async_trait]
    impl UserRepository for StubUserRepo {
        async fn create(&self, _u: &User) -> DomainResult<User> { unimplemented!() }
        async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
            (self.find_by_id_fn.as_ref().unwrap())(id)
        }
        async fn find_by_username(&self, _u: &Username) -> DomainResult<User> { unimplemented!() }
        async fn update(&self, _u: &User) -> DomainResult<User> { unimplemented!() }
        async fn delete(&self, _id: &UserId) -> DomainResult<()> { unimplemented!() }
        async fn list_all(&self, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<User>> { unimplemented!() }
        async fn username_exists(&self, _u: &Username) -> DomainResult<bool> { unimplemented!() }
    }

    // ── Helpers ──────────────────────────────────────────────────

    fn test_org() -> Organization {
        Organization::create(
            OrganizationName::new("Test Org").unwrap(),
            None,
        ).unwrap()
    }

    fn test_user() -> User {
        User::create(
            Username::new("testuser").unwrap(),
            PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap(),
        )
    }

    fn make_uc(
        org_repo: StubOrgRepo,
        user_org_repo: StubUserOrgRepo,
        user_repo: StubUserRepo,
    ) -> OrganizationUseCases<StubOrgRepo, StubUserOrgRepo, StubUserRepo> {
        OrganizationUseCases::new(
            Arc::new(org_repo),
            Arc::new(user_org_repo),
            Arc::new(user_repo),
        )
    }

    // ── Tests ────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_success() {
        let mut repo = StubOrgRepo::default();
        repo.name_exists_fn = Some(Box::new(|_| Ok(false)));
        repo.create_fn = Some(Box::new(|o| Ok(o.clone())));

        let uc = make_uc(repo, StubUserOrgRepo::default(), StubUserRepo::default());
        let name = OrganizationName::new("New Org").unwrap();
        let org = uc.create(name, None).await.unwrap();
        assert_eq!(org.name().as_str(), "New Org");
        assert!(org.is_active());
    }

    #[tokio::test]
    async fn create_duplicate_name() {
        let mut repo = StubOrgRepo::default();
        repo.name_exists_fn = Some(Box::new(|_| Ok(true)));

        let uc = make_uc(repo, StubUserOrgRepo::default(), StubUserRepo::default());
        let name = OrganizationName::new("Existing").unwrap();
        let result = uc.create(name, None).await;
        assert!(matches!(result.unwrap_err(), DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn get_organization() {
        let org = test_org();
        let mut repo = StubOrgRepo::default();
        let o = org.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(o.clone())));

        let uc = make_uc(repo, StubUserOrgRepo::default(), StubUserRepo::default());
        let result = uc.get(org.id()).await.unwrap();
        assert_eq!(result.name().as_str(), "Test Org");
    }

    #[tokio::test]
    async fn update_name() {
        let org = test_org();
        let mut repo = StubOrgRepo::default();
        let o = org.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(o.clone())));
        repo.name_exists_fn = Some(Box::new(|_| Ok(false)));
        repo.update_fn = Some(Box::new(|o| Ok(o.clone())));

        let uc = make_uc(repo, StubUserOrgRepo::default(), StubUserRepo::default());
        let new_name = OrganizationName::new("Updated Org").unwrap();
        let result = uc.update(org.id(), Some(new_name), None).await.unwrap();
        assert_eq!(result.name().as_str(), "Updated Org");
    }

    #[tokio::test]
    async fn update_name_conflict() {
        let org = test_org();
        let mut repo = StubOrgRepo::default();
        let o = org.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(o.clone())));
        repo.name_exists_fn = Some(Box::new(|_| Ok(true)));

        let uc = make_uc(repo, StubUserOrgRepo::default(), StubUserRepo::default());
        let new_name = OrganizationName::new("Taken").unwrap();
        let result = uc.update(org.id(), Some(new_name), None).await;
        assert!(matches!(result.unwrap_err(), DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn toggle_active() {
        let org = test_org();
        assert!(org.is_active());

        let mut repo = StubOrgRepo::default();
        let o = org.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(o.clone())));
        repo.update_fn = Some(Box::new(|o| Ok(o.clone())));

        let uc = make_uc(repo, StubUserOrgRepo::default(), StubUserRepo::default());
        assert!(uc.toggle_active(org.id()).await.is_ok());
    }

    #[tokio::test]
    async fn delete_organization() {
        let org = test_org();
        let mut repo = StubOrgRepo::default();
        let o = org.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(o.clone())));
        repo.delete_fn = Some(Box::new(|_| Ok(())));

        let uc = make_uc(repo, StubUserOrgRepo::default(), StubUserRepo::default());
        assert!(uc.delete(org.id()).await.is_ok());
    }

    #[tokio::test]
    async fn add_user_success() {
        let mut user_org = StubUserOrgRepo::default();
        user_org.is_member_fn = Some(Box::new(|_, _| Ok(false)));
        user_org.add_member_fn = Some(Box::new(|_, _| Ok(())));

        let uc = make_uc(StubOrgRepo::default(), user_org, StubUserRepo::default());
        let uid = UserId::generate();
        let oid = OrganizationId::generate();
        assert!(uc.add_user(&uid, &oid).await.is_ok());
    }

    #[tokio::test]
    async fn add_user_already_member() {
        let mut user_org = StubUserOrgRepo::default();
        user_org.is_member_fn = Some(Box::new(|_, _| Ok(true)));

        let uc = make_uc(StubOrgRepo::default(), user_org, StubUserRepo::default());
        let uid = UserId::generate();
        let oid = OrganizationId::generate();
        let result = uc.add_user(&uid, &oid).await;
        assert!(matches!(result.unwrap_err(), DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn remove_user_success() {
        let mut user_org = StubUserOrgRepo::default();
        user_org.remove_member_fn = Some(Box::new(|_, _| Ok(())));

        let uc = make_uc(StubOrgRepo::default(), user_org, StubUserRepo::default());
        let uid = UserId::generate();
        let oid = OrganizationId::generate();
        assert!(uc.remove_user(&uid, &oid).await.is_ok());
    }

    #[tokio::test]
    async fn list_org_users() {
        let user = test_user();
        let user_id = user.id().clone();

        let mut user_org = StubUserOrgRepo::default();
        let uid = user_id.clone();
        user_org.list_members_fn = Some(Box::new(move |_| {
            let params = PaginationParams::default();
            Ok(PaginatedResult::new(vec![uid.clone()], &params, 1))
        }));

        let mut user_repo = StubUserRepo::default();
        let u = user.clone();
        user_repo.find_by_id_fn = Some(Box::new(move |_| Ok(u.clone())));

        let uc = make_uc(StubOrgRepo::default(), user_org, user_repo);
        let oid = OrganizationId::generate();
        let (users, metadata) = uc.list_users(&oid, None).await.unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(metadata.total_count(), 1);
    }
}
