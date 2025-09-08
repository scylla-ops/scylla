use crate::api::grpc::user::UserRepository;
use pasetors::keys::SymmetricKey;
use pasetors::version4::V4;
use std::sync::Arc;

pub mod controller;
pub mod service;

pub struct AuthService {
    pub(crate) repo: Arc<dyn UserRepository>,
    pub(crate) paseto_secret: SymmetricKey<V4>,
}

impl AuthService {
    pub fn new(repo: Arc<dyn UserRepository>, paseto_secret: SymmetricKey<V4>) -> Self {
        Self {
            repo,
            paseto_secret,
        }
    }
}
