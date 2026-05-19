use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, organization_to_proto,
    proto_to_domain_pagination,
};
use derive_more::Constructor;
use scylla_core::application::OrganizationUseCases;
use scylla_core::application::PermissionService;
use scylla_core::application::{
    OrganizationRepository, UserOrganizationRepository, UserRepository,
};
use scylla_core::domain::entities::{OrganizationId, UserId};
use scylla_core::domain::value_objects::organization::{OrganizationDescription, OrganizationName};
use scylla_core::domain::value_objects::permission::policy;
use scylla_protocol::services::organization::{
    AddUserToOrganizationRequest, AddUserToOrganizationResponse, CreateOrganizationRequest,
    DeleteOrganizationRequest, DeleteOrganizationResponse, GetOrganizationRequest,
    ListOrganizationUsersRequest, ListOrganizationUsersResponse, ListOrganizationsRequest,
    ListOrganizationsResponse, ListUserOrganizationsRequest, ListUserOrganizationsResponse,
    OrganizationResponse, OrganizationUserInfoResponse, RemoveUserFromOrganizationRequest,
    RemoveUserFromOrganizationResponse, ToggleOrganizationActiveRequest,
    ToggleOrganizationActiveResponse, UpdateOrganizationRequest,
    organization_service_server::OrganizationService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct OrganizationHandler<
    O: OrganizationRepository,
    UO: UserOrganizationRepository,
    U: UserRepository,
    PS: PermissionService,
> {
    use_cases: Arc<OrganizationUseCases<O, UO, U>>,
    permission_checker: Arc<PS>,
}

#[async_trait::async_trait]
impl<
    O: OrganizationRepository + Send + Sync + 'static,
    UO: UserOrganizationRepository + Send + Sync + 'static,
    U: UserRepository + Send + Sync + 'static,
    PS: PermissionService + Send + Sync + 'static,
> OrganizationService for OrganizationHandler<O, UO, U, PS>
{
    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        require_permission!(self, request, policy::organization::create());
        let req = request.into_inner();

        let name = OrganizationName::new(&req.name).map_err(domain_error_to_status)?;
        let description = req
            .description
            .map(|d| OrganizationDescription::new(&d))
            .transpose()
            .map_err(domain_error_to_status)?;

        let org = self
            .use_cases
            .create(name, description)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(organization_to_proto(&org)))
    }

    async fn get_organization(
        &self,
        request: Request<GetOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(self, request, policy::organization::get(org_id.clone()));
        let _ = request.into_inner();

        let org = self
            .use_cases
            .get(&org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(organization_to_proto(&org)))
    }

    async fn update_organization(
        &self,
        request: Request<UpdateOrganizationRequest>,
    ) -> Result<Response<OrganizationResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(self, request, policy::organization::update(org_id.clone()));
        let req = request.into_inner();

        let name = req
            .name
            .map(|n| OrganizationName::new(&n))
            .transpose()
            .map_err(domain_error_to_status)?;
        let description = req
            .description
            .map(|d| OrganizationDescription::new(&d).map(Some))
            .transpose()
            .map_err(domain_error_to_status)?;

        let org = self
            .use_cases
            .update(&org_id, name, description)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(organization_to_proto(&org)))
    }

    async fn toggle_organization_active(
        &self,
        request: Request<ToggleOrganizationActiveRequest>,
    ) -> Result<Response<ToggleOrganizationActiveResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(
            self,
            request,
            policy::organization::toggle_active(org_id.clone())
        );
        let _ = request.into_inner();

        self.use_cases
            .toggle_active(&org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ToggleOrganizationActiveResponse {}))
    }

    async fn delete_organization(
        &self,
        request: Request<DeleteOrganizationRequest>,
    ) -> Result<Response<DeleteOrganizationResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(self, request, policy::organization::delete(org_id.clone()));
        let _ = request.into_inner();

        self.use_cases
            .delete(&org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(DeleteOrganizationResponse {}))
    }

    async fn list_organizations(
        &self,
        request: Request<ListOrganizationsRequest>,
    ) -> Result<Response<ListOrganizationsResponse>, Status> {
        require_permission!(self, request, policy::organization::list());
        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let result = self
            .use_cases
            .list(pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let (orgs, metadata) = result.into_parts();
        let organizations: Vec<OrganizationResponse> =
            orgs.iter().map(organization_to_proto).collect();

        Ok(Response::new(ListOrganizationsResponse {
            organizations,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_organization_users(
        &self,
        request: Request<ListOrganizationUsersRequest>,
    ) -> Result<Response<ListOrganizationUsersResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(
            self,
            request,
            policy::organization::list_users(org_id.clone())
        );
        let req = request.into_inner();
        let pagination = proto_to_domain_pagination(req.pagination);

        let (users, metadata) = self
            .use_cases
            .list_users(&org_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let users = users
            .iter()
            .map(|user| OrganizationUserInfoResponse {
                user_id: user.id().to_string(),
                username: user.username().to_string(),
            })
            .collect();

        Ok(Response::new(ListOrganizationUsersResponse {
            users,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn list_user_organizations(
        &self,
        request: Request<ListUserOrganizationsRequest>,
    ) -> Result<Response<ListUserOrganizationsResponse>, Status> {
        let user_id = UserId::new(&request.get_ref().user_id);
        require_permission!(
            self,
            request,
            policy::organization::list_user_orgs(user_id.clone())
        );
        let pagination = proto_to_domain_pagination(request.get_ref().pagination);

        let (orgs, metadata) = self
            .use_cases
            .list_user_orgs(&user_id, pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;

        let organizations: Vec<OrganizationResponse> =
            orgs.iter().map(organization_to_proto).collect();

        Ok(Response::new(ListUserOrganizationsResponse {
            organizations,
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn add_user_to_organization(
        &self,
        request: Request<AddUserToOrganizationRequest>,
    ) -> Result<Response<AddUserToOrganizationResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(
            self,
            request,
            policy::organization::add_user_to_organization(org_id.clone())
        );
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);

        self.use_cases
            .add_user(&user_id, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(AddUserToOrganizationResponse {}))
    }

    async fn remove_user_from_organization(
        &self,
        request: Request<RemoveUserFromOrganizationRequest>,
    ) -> Result<Response<RemoveUserFromOrganizationResponse>, Status> {
        let org_id = OrganizationId::new(&request.get_ref().organization_id);
        require_permission!(
            self,
            request,
            policy::organization::remove_user_from_organization(org_id.clone())
        );
        let req = request.into_inner();
        let user_id = UserId::new(&req.user_id);

        self.use_cases
            .remove_user(&user_id, &org_id)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(RemoveUserFromOrganizationResponse {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_interceptor::AuthContext;
    use async_trait::async_trait;
    use scylla_core::application::OrganizationUseCases;
    use scylla_core::application::PermissionService;
    use scylla_core::application::{
        OrganizationRepository, UserOrganizationRepository, UserRepository,
    };
    use scylla_core::domain::entities::{EntityId, Organization, User};
    use scylla_core::domain::errors::DomainResult;
    use scylla_core::domain::value_objects::organization::{
        OrganizationDescription, OrganizationName,
    };
    use scylla_core::domain::value_objects::permission::policy::{GroupingPolicy, Policy};
    use scylla_core::domain::value_objects::user::{PasswordHash, Username};
    use scylla_core::domain::value_objects::{PaginatedResult, PaginationParams};
    use scylla_protocol::services::organization::organization_service_server::OrganizationService;
    use std::sync::Arc;

    // ── Stubs ──────────────────────────────────────────────────

    #[derive(Default)]
    struct StubOrgRepo {
        create_fn: Option<Box<dyn Fn(&Organization) -> DomainResult<Organization> + Send + Sync>>,
        find_by_id_fn:
            Option<Box<dyn Fn(&OrganizationId) -> DomainResult<Organization> + Send + Sync>>,
        find_by_name_fn:
            Option<Box<dyn Fn(&OrganizationName) -> DomainResult<Organization> + Send + Sync>>,
        update_fn: Option<Box<dyn Fn(&Organization) -> DomainResult<Organization> + Send + Sync>>,
        delete_fn: Option<Box<dyn Fn(&OrganizationId) -> DomainResult<()> + Send + Sync>>,
        list_all_fn:
            Option<Box<dyn Fn() -> DomainResult<PaginatedResult<Organization>> + Send + Sync>>,
        name_exists_fn: Option<Box<dyn Fn(&OrganizationName) -> DomainResult<bool> + Send + Sync>>,
    }

    #[async_trait]
    impl OrganizationRepository for StubOrgRepo {
        async fn create(&self, o: &Organization) -> DomainResult<Organization> {
            (self.create_fn.as_ref().unwrap())(o)
        }
        async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization> {
            (self.find_by_id_fn.as_ref().unwrap())(id)
        }
        async fn find_by_name(&self, n: &OrganizationName) -> DomainResult<Organization> {
            (self.find_by_name_fn.as_ref().unwrap())(n)
        }
        async fn update(&self, o: &Organization) -> DomainResult<Organization> {
            (self.update_fn.as_ref().unwrap())(o)
        }
        async fn delete(&self, id: &OrganizationId) -> DomainResult<()> {
            (self.delete_fn.as_ref().unwrap())(id)
        }
        async fn list_all(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Organization>> {
            (self.list_all_fn.as_ref().unwrap())()
        }
        async fn list_active(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<Organization>> {
            unimplemented!()
        }
        async fn name_exists(&self, n: &OrganizationName) -> DomainResult<bool> {
            (self.name_exists_fn.as_ref().unwrap())(n)
        }
    }

    #[derive(Default)]
    struct StubUserOrgRepo {
        add_member_fn:
            Option<Box<dyn Fn(&UserId, &OrganizationId) -> DomainResult<()> + Send + Sync>>,
        remove_member_fn:
            Option<Box<dyn Fn(&UserId, &OrganizationId) -> DomainResult<()> + Send + Sync>>,
        is_member_fn:
            Option<Box<dyn Fn(&UserId, &OrganizationId) -> DomainResult<bool> + Send + Sync>>,
        list_members_fn: Option<
            Box<dyn Fn(&OrganizationId) -> DomainResult<PaginatedResult<UserId>> + Send + Sync>,
        >,
        list_user_orgs_fn: Option<
            Box<dyn Fn(&UserId) -> DomainResult<PaginatedResult<OrganizationId>> + Send + Sync>,
        >,
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
        async fn list_members(
            &self,
            oid: &OrganizationId,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<UserId>> {
            (self.list_members_fn.as_ref().unwrap())(oid)
        }
        async fn list_user_organizations(
            &self,
            uid: &UserId,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<OrganizationId>> {
            (self.list_user_orgs_fn.as_ref().unwrap())(uid)
        }
    }

    #[derive(Default)]
    struct StubUserRepo {
        find_by_id_fn: Option<Box<dyn Fn(&UserId) -> DomainResult<User> + Send + Sync>>,
    }

    #[async_trait]
    impl UserRepository for StubUserRepo {
        async fn create(&self, _u: &User) -> DomainResult<User> {
            unimplemented!()
        }
        async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
            (self.find_by_id_fn.as_ref().unwrap())(id)
        }
        async fn find_by_username(&self, _u: &Username) -> DomainResult<User> {
            unimplemented!()
        }
        async fn update(&self, _u: &User) -> DomainResult<User> {
            unimplemented!()
        }
        async fn delete(&self, _id: &UserId) -> DomainResult<()> {
            unimplemented!()
        }
        async fn list_all(
            &self,
            _p: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<User>> {
            unimplemented!()
        }
        async fn username_exists(&self, _u: &Username) -> DomainResult<bool> {
            unimplemented!()
        }
    }

    struct AllowAll;

    #[async_trait]
    impl PermissionService for AllowAll {
        async fn check(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> {
            Ok(true)
        }
        async fn add_policy(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> {
            Ok(true)
        }
        async fn remove_policy(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> {
            Ok(true)
        }
        async fn list_policies(&self, _s: Option<&str>) -> DomainResult<Vec<(String, Policy)>> {
            Ok(vec![])
        }
        async fn add_grouping_policy(
            &self,
            _s: impl EntityId,
            _p: GroupingPolicy,
        ) -> DomainResult<bool> {
            Ok(true)
        }
        async fn remove_grouping_policy(
            &self,
            _s: impl EntityId,
            _p: GroupingPolicy,
        ) -> DomainResult<bool> {
            Ok(true)
        }
        async fn list_grouping_policies(
            &self,
            _s: Option<&str>,
        ) -> DomainResult<Vec<(String, GroupingPolicy)>> {
            Ok(vec![])
        }
    }

    // ── Helpers ─────────────────────────────────────────────────

    fn test_org() -> Organization {
        Organization::create(
            OrganizationName::new("testorg").unwrap(),
            Some(OrganizationDescription::new("A test org").unwrap()),
        )
        .unwrap()
    }

    fn authed_request<T>(body: T) -> Request<T> {
        let mut req = Request::new(body);
        req.extensions_mut()
            .insert(AuthContext::new(UserId::generate()));
        req
    }

    fn make_handler(
        org_repo: StubOrgRepo,
        user_org_repo: StubUserOrgRepo,
        user_repo: StubUserRepo,
    ) -> OrganizationHandler<StubOrgRepo, StubUserOrgRepo, StubUserRepo, AllowAll> {
        let uc = Arc::new(OrganizationUseCases::new(
            Arc::new(org_repo),
            Arc::new(user_org_repo),
            Arc::new(user_repo),
        ));
        OrganizationHandler::new(uc, Arc::new(AllowAll))
    }

    // ── Tests ───────────────────────────────────────────────────

    #[tokio::test]
    async fn create_organization_returns_proto() {
        let mut repo = StubOrgRepo::default();
        repo.name_exists_fn = Some(Box::new(|_| Ok(false)));
        repo.create_fn = Some(Box::new(|o| Ok(o.clone())));

        let handler = make_handler(repo, StubUserOrgRepo::default(), StubUserRepo::default());
        let req = authed_request(CreateOrganizationRequest {
            name: "neworg".into(),
            description: Some("desc".into()),
        });

        let resp = handler.create_organization(req).await.unwrap();
        assert_eq!(resp.into_inner().name, "neworg");
    }

    #[tokio::test]
    async fn get_organization_returns_proto() {
        let org = test_org();
        let org_id_str = org.id().to_string();
        let o = org.clone();

        let mut repo = StubOrgRepo::default();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(o.clone())));

        let handler = make_handler(repo, StubUserOrgRepo::default(), StubUserRepo::default());
        let req = authed_request(GetOrganizationRequest {
            organization_id: org_id_str,
        });

        let resp = handler.get_organization(req).await.unwrap();
        assert_eq!(resp.into_inner().name, "testorg");
    }

    #[tokio::test]
    async fn delete_organization_success() {
        let org = test_org();
        let org_id_str = org.id().to_string();

        let mut repo = StubOrgRepo::default();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(org.clone())));
        repo.delete_fn = Some(Box::new(|_| Ok(())));

        let handler = make_handler(repo, StubUserOrgRepo::default(), StubUserRepo::default());
        let req = authed_request(DeleteOrganizationRequest {
            organization_id: org_id_str,
        });
        assert!(handler.delete_organization(req).await.is_ok());
    }

    #[tokio::test]
    async fn list_organizations_returns_empty() {
        let mut repo = StubOrgRepo::default();
        repo.list_all_fn = Some(Box::new(|| {
            let params = PaginationParams::default();
            Ok(PaginatedResult::new(vec![], &params, 0))
        }));

        let handler = make_handler(repo, StubUserOrgRepo::default(), StubUserRepo::default());
        let req = authed_request(ListOrganizationsRequest { pagination: None });

        let resp = handler.list_organizations(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.organizations.is_empty());
        assert!(inner.pagination.is_some());
    }

    #[tokio::test]
    async fn add_user_to_organization_success() {
        let org = test_org();
        let org_id_str = org.id().to_string();
        let user = User::create(
            Username::new("member").unwrap(),
            PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap(),
        );
        let user_id_str = user.id().to_string();

        let mut user_org_repo = StubUserOrgRepo::default();
        user_org_repo.is_member_fn = Some(Box::new(|_, _| Ok(false)));
        user_org_repo.add_member_fn = Some(Box::new(|_, _| Ok(())));

        let handler = make_handler(
            StubOrgRepo::default(),
            user_org_repo,
            StubUserRepo::default(),
        );
        let req = authed_request(AddUserToOrganizationRequest {
            organization_id: org_id_str,
            user_id: user_id_str,
        });

        assert!(handler.add_user_to_organization(req).await.is_ok());
    }

    #[tokio::test]
    async fn remove_user_from_organization_success() {
        let org = test_org();
        let org_id_str = org.id().to_string();
        let user_id = UserId::generate();

        let mut user_org_repo = StubUserOrgRepo::default();
        user_org_repo.remove_member_fn = Some(Box::new(|_, _| Ok(())));

        let handler = make_handler(
            StubOrgRepo::default(),
            user_org_repo,
            StubUserRepo::default(),
        );
        let req = authed_request(RemoveUserFromOrganizationRequest {
            organization_id: org_id_str,
            user_id: user_id.to_string(),
        });

        assert!(handler.remove_user_from_organization(req).await.is_ok());
    }

    #[tokio::test]
    async fn create_organization_without_auth_fails() {
        let handler = make_handler(
            StubOrgRepo::default(),
            StubUserOrgRepo::default(),
            StubUserRepo::default(),
        );
        let req = Request::new(CreateOrganizationRequest {
            name: "org".into(),
            description: None,
        });

        let err = handler.create_organization(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Internal);
    }
}
