use crate::api::v1::modules::teams::repository::TeamRepository;

pub struct TeamService {
    repository: TeamRepository,
}

impl TeamService {
    pub fn new(repository: TeamRepository) -> Self {
        Self { repository }
    }
}
