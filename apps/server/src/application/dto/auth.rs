use crate::domain::value_objects::{Password, UserId, Username};

#[derive(Debug, Clone)]
pub struct LoginRequestDto {
    pub username: Username,
    pub password: Password,
}

#[derive(Debug, Clone)]
pub struct LoginResponseDto {
    pub token: String,
    pub user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct ValidateTokenRequestDto {
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct ValidateTokenResponseDto {
    pub is_valid: bool,
}

#[derive(Debug, Clone)]
pub struct RevokeTokenRequestDto {
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct RevokeTokenResponseDto {}
