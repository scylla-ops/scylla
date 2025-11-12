use crate::application::dto::{
    ListUserOrganizationsRequestDto, ListUserOrganizationsResponseDto, OrganizationResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::{OrganizationRepository, UserOrganizationRepository};
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct ListUserOrganizationsUseCase<UOR, OR>
where
    UOR: UserOrganizationRepository + ?Sized,
    OR: OrganizationRepository + ?Sized,
{
    user_org_repo: Arc<UOR>,
    organization_repo: Arc<OR>,
}

impl<UOR, OR> ListUserOrganizationsUseCase<UOR, OR>
where
    UOR: UserOrganizationRepository + ?Sized,
    OR: OrganizationRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: ListUserOrganizationsRequestDto,
    ) -> DomainResult<ListUserOrganizationsResponseDto> {
        let paginated_result = self
            .user_org_repo
            .list_organizations_for_user(&request.user_id, request.pagination.as_ref())
            .await?;

        let (organization_ids, metadata) = paginated_result.into_parts();

        let mut organizations = Vec::new();
        for org_id in organization_ids {
            let organization = self.organization_repo.find_by_id(&org_id).await?;
            organizations.push(organization);
        }

        Ok(ListUserOrganizationsResponseDto {
            organizations: organizations
                .into_iter()
                .map(OrganizationResponseDto::from)
                .collect(),
            pagination: Some(metadata),
        })
    }
}
