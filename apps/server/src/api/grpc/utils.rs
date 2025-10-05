use argon2::password_hash::{PasswordHash, SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};

pub(crate) type Id = String;

pub fn hash_password(password: impl AsRef<[u8]>) -> anyhow::Result<String> {
    let salt = SaltString::try_from_rng(&mut OsRng)?;
    let argon2 = Argon2::default(); // todo: use better config for production
    let password_hash = argon2.hash_password(password.as_ref(), &salt)?;
    Ok(password_hash.to_string())
}

pub fn verify_password(password: impl AsRef<[u8]>, hash: &str) -> anyhow::Result<bool> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(argon2
        .verify_password(password.as_ref(), &parsed_hash)
        .is_ok())
}
