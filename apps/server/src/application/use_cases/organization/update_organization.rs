use crate::application::dto::{OrganizationResponseDto, UpdateOrganizationRequestDto};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::OrganizationRepository;
use std::sync::Arc;

pub struct UpdateOrganizationUseCase<R>
where
    R: OrganizationRepository + ?Sized,
{
    org_repo: Arc<R>,
}

impl<R> UpdateOrganizationUseCase<R>
where
    R: OrganizationRepository + ?Sized,
{
    pub fn new(org_repo: Arc<R>) -> Self {
        Self { org_repo }
    }

    pub async fn execute(
        &self,
        request: UpdateOrganizationRequestDto,
    ) -> DomainResult<OrganizationResponseDto> {
        let mut organization_draft = self.org_repo.find_by_id(&request.organization_id).await?;

        if let Some(name) = request.name {
            // Check if new name is taken by another organization
            if self.org_repo.name_exists(&name).await? {
                // Get the existing org to check if it's the same one we're updating
                let existing_org = self.org_repo.find_by_name(&name).await?;
                if existing_org.id() != &request.organization_id {
                    return Err(DomainError::conflict(format!(
                        "Organization name '{}' is already taken",
                        name
                    )));
                }
            }
            organization_draft.update_name(name)?;
        }

        if let Some(description) = request.description {
            organization_draft.update_description(Some(description))?;
        }

        let updated_organization = self.org_repo.update(&organization_draft).await?;

        Ok(OrganizationResponseDto::from(updated_organization))
    }
}
