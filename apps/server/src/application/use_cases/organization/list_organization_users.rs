use crate::application::dto::{
    ListOrganizationUsersRequestDto, ListOrganizationUsersResponseDto, UserInfoDto,
};
use crate::domain::entities::{User, UserOrganization};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::{UserOrganizationRepository, UserRepository};
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct ListOrganizationUsersUseCase<UOR, UR>
where
    UOR: UserOrganizationRepository + ?Sized,
    UR: UserRepository + ?Sized,
{
    user_org_repo: Arc<UOR>,
    user_repo: Arc<UR>,
}

impl<UOR, UR> ListOrganizationUsersUseCase<UOR, UR>
where
    UOR: UserOrganizationRepository + ?Sized,
    UR: UserRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: ListOrganizationUsersRequestDto,
    ) -> DomainResult<ListOrganizationUsersResponseDto> {
        let paginated_result = self
            .user_org_repo
            .list_users_in_organization(&request.organization_id, request.pagination.as_ref())
            .await?;

        let (user_ids, metadata) = paginated_result.into_parts();

        let mut users = Vec::new();
        for user_id in user_ids {
            let user: User = self.user_repo.find_by_id(&user_id).await?;

            let user_org: UserOrganization = self
                .user_org_repo
                .find_by_user_and_organization(&user_id, &request.organization_id)
                .await?;

            users.push(UserInfoDto {
                user_id: user.id().clone(),
                username: user.username().to_string(),
                role: user_org.role().clone(),
                joined_at: user_org.joined_at(),
            });
        }

        Ok(ListOrganizationUsersResponseDto {
            users,
            pagination: Some(metadata),
        })
    }
}
