use crate::application::dto::{
    ToggleActiveOrganizationRequestDto, ToggleActiveOrganizationResponseDto,
};
use crate::domain::errors::DomainResult;
use crate::domain::repositories::OrganizationRepository;
use derive_more::Constructor;
use std::sync::Arc;

#[derive(Constructor)]
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
