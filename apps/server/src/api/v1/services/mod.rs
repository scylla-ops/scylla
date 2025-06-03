// Import and re-export services from submodules
mod command;

// Re-export the CommandService for backward compatibility
pub use command::CommandService;
