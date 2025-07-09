use crate::api::v1::common::base::{BaseRepository, Repository};
use crate::database::DieselPool;

pub struct TeamRepository {
    base: BaseRepository,
}

impl TeamRepository {
    pub fn new(pool: DieselPool) -> Self {
        Self {
            base: BaseRepository::new(pool),
        }
    }
}

// Implement Repository trait for CommandRepository by delegating to the base repository
impl Repository for TeamRepository {
    fn get_pool(&self) -> &DieselPool {
        self.base.get_pool()
    }
}
