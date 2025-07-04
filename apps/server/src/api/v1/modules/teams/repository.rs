use crate::api::v1::common::base::BaseRepository;
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
