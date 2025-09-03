use crate::api::grpc::user::UserRepository;
use pasetors::keys::SymmetricKey;
use pasetors::version4::V4;
use std::sync::Arc;

pub mod service;

pub struct AuthService {
    repo: Arc<dyn UserRepository>,
    paseto_secret: SymmetricKey<V4>,
}

impl AuthService {
    pub fn new(repo: Arc<dyn UserRepository>, paseto_secret: SymmetricKey<V4>) -> Self {
        Self {
            repo,
            paseto_secret,
        }
    }
}
