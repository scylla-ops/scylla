use crate::domain::entities::Organization;
use crate::domain::value_objects::{
    Description, OrganizationId, OrganizationName, PaginationMetadata, PaginationParams, UserId,
    UserOrganizationId, UserOrganizationRole,
};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct CreateOrganizationRequestDto {
    pub name: OrganizationName,
    pub description: Option<Description>,
    pub creator_id: UserId,
}

#[derive(Debug, Clone)]
pub struct GetOrganizationRequestDto {
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct UpdateOrganizationRequestDto {
    pub organization_id: OrganizationId,
    pub name: Option<OrganizationName>,
    pub description: Option<Description>,
}

#[derive(Debug, Clone)]
pub struct ToggleActiveOrganizationRequestDto {
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct OrganizationResponseDto {
    pub id: OrganizationId,
    pub name: OrganizationName,
    pub description: Option<Description>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Organization> for OrganizationResponseDto {
    fn from(org: Organization) -> Self {
        Self {
            id: org.id().to_owned(),
            name: org.name().to_owned(),
            description: org.description().map(|d| d.to_owned()),
            is_active: org.is_active(),
            created_at: org.created_at(),
            updated_at: org.updated_at(),
        }
    }
}

impl From<&Organization> for OrganizationResponseDto {
    fn from(org: &Organization) -> Self {
        Self {
            id: org.id().to_owned(),
            name: org.name().to_owned(),
            description: org.description().map(|d| d.to_owned()),
            is_active: org.is_active(),
            created_at: org.created_at(),
            updated_at: org.updated_at(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToggleOrganizationActiveRequestDto {
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct ToggleActiveOrganizationResponseDto {}

#[derive(Debug, Clone)]
pub struct DeleteOrganizationRequestDto {
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct DeleteOrganizationResponseDto {}

#[derive(Debug, Clone)]
pub struct ListOrganizationsRequestDto {
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone)]
pub struct ListOrganizationsResponseDto {
    pub organizations: Vec<OrganizationResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}

#[derive(Debug, Clone)]
pub struct ListOrganizationUsersRequestDto {
    pub organization_id: OrganizationId,
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone)]
pub struct UserInfoDto {
    pub user_id: UserId,
    pub username: String,
    pub role: UserOrganizationRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ListOrganizationUsersResponseDto {
    pub users: Vec<UserInfoDto>,
    pub pagination: Option<PaginationMetadata>,
}

#[derive(Debug, Clone)]
pub struct ListUserOrganizationsRequestDto {
    pub user_id: UserId,
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone)]
pub struct ListUserOrganizationsResponseDto {
    pub organizations: Vec<OrganizationResponseDto>,
    pub pagination: Option<PaginationMetadata>,
}

#[derive(Debug, Clone)]
pub struct AddUserToOrganizationRequestDto {
    pub user_id: UserId,
    pub organization_id: OrganizationId,
    pub role: UserOrganizationRole,
}

#[derive(Debug, Clone)]
pub struct AddUserToOrganizationResponseDto {
    pub relation_id: UserOrganizationId,
}

#[derive(Debug, Clone)]
pub struct RemoveUserFromOrganizationRequestDto {
    pub user_id: UserId,
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct RemoveUserFromOrganizationResponseDto {}
