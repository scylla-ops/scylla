//! `User` test fixtures.

use bon::bon;
use chrono::{DateTime, Utc};

use crate::domain::entities::{User, UserId};
use crate::domain::value_objects::user::{PasswordHash, Username};

/// A PHC-format Argon2id hash that satisfies `PasswordHash::new`. Constant so
/// builders don't pay the hashing cost in tests.
pub const VALID_ARGON2_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$abc$xyz";

pub struct UserBuilder;

#[bon]
#[allow(clippy::new_ret_no_self, clippy::must_use_candidate)]
impl UserBuilder {
    #[builder(start_fn = new, finish_fn = build)]
    pub fn assemble(
        #[builder(start_fn, into)] username: String,
        id: Option<UserId>,
        #[builder(into, default = VALID_ARGON2_HASH.to_string())] password_hash: String,
        #[builder(default = true)] is_active: bool,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> User {
        let now = created_at.unwrap_or_else(Utc::now);
        User::from_persistence(
            id.unwrap_or_else(UserId::generate),
            Username::new(username).expect("test username invalid"),
            PasswordHash::new(password_hash).expect("test password hash invalid"),
            is_active,
            now,
            updated_at.unwrap_or(now),
        )
    }
}

/// Shortcut for "give me a fresh valid user named X".
#[must_use]
pub fn user(name: &str) -> User {
    UserBuilder::new(name).build()
}

#[cfg(feature = "postgres")]
pub async fn seed_user(pool: &sqlx::PgPool, name: &str) -> User {
    use crate::application::ports::UserRepository;
    use crate::infrastructure::persistence::postgres::PgUserRepository;
    let user = user(name);
    PgUserRepository::new(pool.clone())
        .create(&user)
        .await
        .expect("seed user failed");
    user
}
