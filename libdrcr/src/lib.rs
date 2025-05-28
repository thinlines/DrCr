pub mod account_config;
pub mod db;
pub mod model;
pub mod reporting;
pub mod serde;
pub mod util;

/// Data type used to represent transaction and account quantities
pub type QuantityInt = i64;

// Magic strings
// TODO: Make this configurable
pub const CURRENT_YEAR_EARNINGS: &'static str = "Current Year Earnings";
pub const RETAINED_EARNINGS: &'static str = "Retained Earnings";
