mod service;
pub mod types;

// ---------------------------------------------------------------------------
// Re-exports for backward compatibility
// ---------------------------------------------------------------------------

pub use service::{AccountService, AccountServiceImpl};
pub use types::{
    AccountListQuery, CreateAccountRequest, FireflyAccountResource, UpdateAccountRequest,
};
pub use types::{DEFAULT_LIMIT, DEFAULT_PAGE, MAX_LIMIT};
