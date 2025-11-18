use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::get_data_dir;

/// User profile containing authentication information
/// Stored in the system's data directory using the `directories` crate
///
/// Location varies by platform:
/// - Linux: `~/.local/share/scylla/stui/profiles.json`
/// - macOS: `~/Library/Application Support/scylla/stui/profiles.json`
/// - Windows: `%APPDATA%\scylla\stui\profiles.json`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfile {
    pub username: String,
}

impl UserProfile {
    pub fn new(username: String) -> Self {
        Self { username }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileStorage {
    profiles: Vec<UserProfile>,
}

impl ProfileStorage {
    /// Load profiles from disk using the `directories` crate
    ///
    /// The file is stored in the platform-specific data directory
    /// determined by `ProjectDirs` from the `directories` crate
    pub fn load() -> Result<Self> {
        let path = Self::profiles_file_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)?;
        let storage: ProfileStorage = serde_json::from_str(&contents)?;
        Ok(storage)
    }

    /// Save profiles to disk using the `directories` crate
    ///
    /// Creates the data directory if it doesn't exist
    pub fn save(&self) -> Result<()> {
        let path = Self::profiles_file_path()?;

        // Create parent directories if they don't exist
        // This uses the platform-specific location from `directories` crate
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        Ok(())
    }

    /// Get the file path for storing profiles
    ///
    /// Uses `get_data_dir()` which internally uses the `directories` crate
    /// via `ProjectDirs` to determine the platform-specific data directory
    fn profiles_file_path() -> Result<PathBuf> {
        let data_dir = get_data_dir();
        Ok(data_dir.join("profiles.json"))
    }

    /// Get all saved profiles
    pub fn get_profiles(&self) -> &[UserProfile] {
        &self.profiles
    }

    /// Add or update a profile (ensures username is unique)
    pub fn add_profile(&mut self, profile: UserProfile) {
        // Remove existing profile with same username if it exists
        self.profiles.retain(|p| p.username != profile.username);

        // Add new profile at the beginning (most recent first)
        self.profiles.insert(0, profile);

        // Keep only the last 10 profiles
        if self.profiles.len() > 10 {
            self.profiles.truncate(10);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_profile() {
        let mut storage = ProfileStorage::default();
        storage.add_profile(UserProfile::new("user1".to_string()));

        assert_eq!(storage.get_profiles().len(), 1);
        assert_eq!(storage.get_profiles().first().unwrap().username, "user1");
    }

    #[test]
    fn test_unique_usernames() {
        let mut storage = ProfileStorage::default();
        storage.add_profile(UserProfile::new("user1".to_string()));
        storage.add_profile(UserProfile::new("user2".to_string()));
        storage.add_profile(UserProfile::new("user1".to_string()));

        // Should only have 2 profiles (user1 updated, not duplicated)
        assert_eq!(storage.get_profiles().len(), 2);
        assert_eq!(storage.get_profiles().first().unwrap().username, "user1");
    }

    #[test]
    fn test_max_profiles() {
        let mut storage = ProfileStorage::default();
        for i in 0..15 {
            storage.add_profile(UserProfile::new(format!("user{}", i)));
        }

        // Should only keep last 10
        assert_eq!(storage.get_profiles().len(), 10);
    }
}
