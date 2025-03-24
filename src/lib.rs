// Expose our E/C/S modules to users
pub mod e;
pub mod c;
pub mod s;

// Re-export the core types for convenience
pub use e::entity::World;
pub use e::factory::HexMapFactory; 