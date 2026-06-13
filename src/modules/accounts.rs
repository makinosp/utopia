mod service;
pub mod types;

// Re-exports for backward compatibility
pub use service::{AccountService, AccountServiceImpl};
pub use types::{
    AccountListQuery, CreateAccountRequest, FireflyAccountResource, UpdateAccountRequest,
};
// These are used in tests, so we allow unused_imports to avoid warnings in the main binary
#[allow(unused_imports)]
pub use types::{DEFAULT_LIMIT, DEFAULT_PAGE, MAX_LIMIT};
