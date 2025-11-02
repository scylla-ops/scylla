use crate::api::grpc::organization::models::{
    InsertableOrganization, Organization, OrganizationPatch, UserOrganizationRelation,
};
use crate::api::grpc::organization::repos::OrganizationRepository;
use crate::api::grpc::user::models::User;
use crate::api::grpc::user::repos::UserRepository;
use derive_more::Constructor;
use surrealdb::RecordIdKey;
use thiserror::Error;

#[derive(Constructor)]
pub struct OrganizationService<R: OrganizationRepository, UR: UserRepository> {
    _marker: std::marker::PhantomData<(R, UR)>,
}

#[derive(Debug, Error)]
pub enum OrganizationDomainError {
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Invalid pagination parameter: {field}")]
    InvalidPagination { field: &'static str },
    #[error("Organization not found")]
    OrganizationNotFound,
    #[error("User not found")]
    UserNotFound,
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

impl<R: OrganizationRepository, UR: UserRepository> OrganizationService<R, UR> {
    pub async fn create_organization(
        name: String,
        description: Option<String>,
    ) -> Result<Organization, OrganizationDomainError> {
        let organization = InsertableOrganization { name, description };

        R::create_organization(organization)
            .await
            .map_err(OrganizationDomainError::Repo)
    }

    pub async fn get_organization(
        org_id: RecordIdKey,
    ) -> Result<Organization, OrganizationDomainError> {
        let opt = R::get_organization_by_id(org_id).await?;
        match opt {
            Some(org) => Ok(org),
            None => Err(OrganizationDomainError::OrganizationNotFound),
        }
    }

    pub async fn list_organizations(
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Organization>, usize), OrganizationDomainError> {
        // validate pagination parameters
        const MAX_PAGE_SIZE: u32 = 100;
        if page == 0 {
            return Err(OrganizationDomainError::InvalidPagination { field: "page" });
        }
        if page_size == 0 {
            return Err(OrganizationDomainError::InvalidPagination { field: "page_size" });
        }
        if page_size > MAX_PAGE_SIZE {
            return Err(OrganizationDomainError::Validation(format!(
                "page_size must be <= {}",
                MAX_PAGE_SIZE
            )));
        }

        // calculate limit and offset
        let limit_i64: i64 = page_size.into();
        let offset_u128 = (page as u128)
            .saturating_sub(1)
            .saturating_mul(page_size as u128);
        let offset_i64 = i64::try_from(offset_u128).map_err(|_| {
            OrganizationDomainError::Validation("computed offset is too big".into())
        })?;

        let list = R::list_organizations(limit_i64, offset_i64).await?;
        let total = list.len();
        Ok((list, total))
    }

    pub async fn update_organization(
        org_id: RecordIdKey,
        patch: OrganizationPatch,
    ) -> Result<Organization, OrganizationDomainError> {
        let opt = R::update_organization(org_id, patch).await?;

        match opt {
            Some(org) => Ok(org),
            None => Err(OrganizationDomainError::OrganizationNotFound),
        }
    }

    pub async fn deactivate_organization(
        org_id: RecordIdKey,
    ) -> Result<(), OrganizationDomainError> {
        let res = R::deactivate_organization(org_id).await?;
        match res {
            Some(_) => Ok(()),
            None => Err(OrganizationDomainError::OrganizationNotFound),
        }
    }

    pub async fn add_user_to_organization(
        user_id: RecordIdKey,
        org_id: RecordIdKey,
        role: String,
    ) -> Result<UserOrganizationRelation, OrganizationDomainError> {
        // verify user exists
        let _user = UR::get_user_by_id(user_id.to_string())
            .await?
            .ok_or(OrganizationDomainError::UserNotFound)?;

        // verify organization exists
        let _org = R::get_organization_by_id(org_id.clone())
            .await?
            .ok_or(OrganizationDomainError::OrganizationNotFound)?;

        R::add_user_to_organization(user_id, org_id, role)
            .await
            .map_err(OrganizationDomainError::Repo)
    }

    pub async fn remove_user_from_organization(
        user_id: RecordIdKey,
        org_id: RecordIdKey,
    ) -> Result<(), OrganizationDomainError> {
        R::remove_user_from_organization(user_id, org_id)
            .await
            .map_err(OrganizationDomainError::Repo)
    }

    pub async fn list_organization_users(
        org_id: RecordIdKey,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<(User, UserOrganizationRelation)>, usize), OrganizationDomainError> {
        // verify organization exists
        let _ = R::get_organization_by_id(org_id.clone())
            .await?
            .ok_or(OrganizationDomainError::OrganizationNotFound)?;

        // validate pagination parameters
        const MAX_PAGE_SIZE: u32 = 100;
        if page == 0 {
            return Err(OrganizationDomainError::InvalidPagination { field: "page" });
        }
        if page_size == 0 {
            return Err(OrganizationDomainError::InvalidPagination { field: "page_size" });
        }
        if page_size > MAX_PAGE_SIZE {
            return Err(OrganizationDomainError::Validation(format!(
                "page_size must be <= {}",
                MAX_PAGE_SIZE
            )));
        }

        // calculate limit and offset
        let limit: i64 = page_size.into();
        let offset_u128 = (page as u128)
            .saturating_sub(1)
            .saturating_mul(page_size as u128);
        let offset: i64 = i64::try_from(offset_u128).map_err(|_| {
            OrganizationDomainError::Validation("computed offset is too big".into())
        })?;

        let list = R::list_organization_users(org_id, limit, offset).await?;
        let total = list.len();
        Ok((list, total))
    }

    pub async fn list_user_organizations(
        user_id: RecordIdKey,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<(Organization, UserOrganizationRelation)>, usize), OrganizationDomainError>
    {
        // verify user exists
        let _user = UR::get_user_by_id(user_id.to_string())
            .await?
            .ok_or(OrganizationDomainError::UserNotFound)?;

        // validate pagination parameters
        const MAX_PAGE_SIZE: u32 = 100;
        if page == 0 {
            return Err(OrganizationDomainError::InvalidPagination { field: "page" });
        }
        if page_size == 0 {
            return Err(OrganizationDomainError::InvalidPagination { field: "page_size" });
        }
        if page_size > MAX_PAGE_SIZE {
            return Err(OrganizationDomainError::Validation(format!(
                "page_size must be <= {}",
                MAX_PAGE_SIZE
            )));
        }

        // calculate limit and offset
        let limit: i64 = page_size.into();
        let offset_u128 = (page as u128)
            .saturating_sub(1)
            .saturating_mul(page_size as u128);
        let offset: i64 = i64::try_from(offset_u128).map_err(|_| {
            OrganizationDomainError::Validation("computed offset is too big".into())
        })?;

        let list = R::list_user_organizations(user_id, limit, offset).await?;
        let total = list.len();
        Ok((list, total))
    }
}
