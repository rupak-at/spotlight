pub mod config;
pub mod files;
pub mod model;
pub mod repository;
pub mod search;

pub type Result<T> = std::result::Result<T, String>;
