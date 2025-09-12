pub mod auth;
mod job;
pub mod orchestrator;
pub mod pipeline;
pub mod user;
pub mod utils;

pub use user::repo::UserRepositoryDiesel;

pub use auth::AuthService;

use tokio::{sync::watch, task::JoinHandle};

pub trait BackgroundWorker: Send + Sync + 'static {
    fn spawn_worker(self, shutdown: watch::Receiver<bool>) -> JoinHandle<()>;
}
