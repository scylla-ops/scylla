use crate::application::dto::{
    ListOrganizationsRequestDto, ListOrganizationsResponseDto, OrganizationResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::OrganizationRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
pub struct ListOrganizationsUseCase<R>
where
    R: OrganizationRepository + ?Sized,
{
    org_repo: Arc<R>,
}

impl<R> ListOrganizationsUseCase<R>
where
    R: OrganizationRepository + ?Sized,
{
    pub async fn execute(
        &self,
        request: ListOrganizationsRequestDto,
    ) -> DomainResult<ListOrganizationsResponseDto> {
        let paginated_result = self.org_repo.list_all(request.pagination.as_ref()).await?;
        let (organizations, metadata) = paginated_result.into_parts();

        Ok(ListOrganizationsResponseDto {
            organizations: organizations
                .into_iter()
                .map(OrganizationResponseDto::from)
                .collect(),
            pagination: Some(metadata),
        })
    }
}
