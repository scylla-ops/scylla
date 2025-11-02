use crate::application::dto::{
    ListOrganizationsRequestDto, ListOrganizationsResponseDto, OrganizationResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::OrganizationRepository;
use std::sync::Arc;

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
    pub fn new(org_repo: Arc<R>) -> Self {
        Self { org_repo }
    }

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
