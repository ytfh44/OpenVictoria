// Expose our E/C/S modules to users
pub mod e;
pub mod c;
pub mod s;
pub mod i18n;

// Re-export the core types for convenience
pub use e::entity::World;
pub use e::factory::HexMapFactory;
pub use i18n::Locale; 