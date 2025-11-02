use crate::application::dto::{
    ToggleActiveOrganizationRequestDto, ToggleActiveOrganizationResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::OrganizationRepository;
use std::sync::Arc;

pub struct ToggleActiveOrganizationUseCase<R>
where
    R: OrganizationRepository + ?Sized,
{
    org_repo: Arc<R>,
}

impl<R> ToggleActiveOrganizationUseCase<R>
where
    R: OrganizationRepository + ?Sized,
{
    pub fn new(org_repo: Arc<R>) -> Self {
        Self { org_repo }
    }

    pub async fn execute(
        &self,
        request: ToggleActiveOrganizationRequestDto,
    ) -> DomainResult<ToggleActiveOrganizationResponseDto> {
        let mut organization_draft = self.org_repo.find_by_id(&request.organization_id).await?;

        organization_draft.toggle_active()?;

        self.org_repo.update(&organization_draft).await?;

        Ok(ToggleActiveOrganizationResponseDto {})
    }
}
