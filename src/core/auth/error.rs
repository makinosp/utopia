use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum AuthError {
    MissingAuthorizationHeader,
    TokenMalformed,
    TokenNotFound,
    TokenRevoked,
    UserBlocked,
    DependencyFailure,
    BootstrapKeyMissing,
    BootstrapKeyInvalid,
    BootstrapAlreadyUsed,
}

impl AuthError {
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::MissingAuthorizationHeader => "unauthenticated",
            Self::TokenMalformed => "token_malformed",
            Self::TokenNotFound => "token_not_found",
            Self::TokenRevoked => "token_revoked",
            Self::UserBlocked => "user_blocked",
            Self::DependencyFailure => "dependency_failure",
            Self::BootstrapKeyMissing => "bootstrap_key_missing",
            Self::BootstrapKeyInvalid => "bootstrap_key_invalid",
            Self::BootstrapAlreadyUsed => "bootstrap_key_already_used",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::MissingAuthorizationHeader => "Missing or invalid bearer token.",
            Self::TokenMalformed => "The token format is invalid.",
            Self::TokenNotFound => "The provided token was not found.",
            Self::TokenRevoked => "The provided token has been revoked.",
            Self::UserBlocked => "The associated user is blocked.",
            Self::DependencyFailure => "Authentication dependency is unavailable.",
            Self::BootstrapKeyMissing => "Bootstrap key is required.",
            Self::BootstrapKeyInvalid => "Bootstrap key is invalid.",
            Self::BootstrapAlreadyUsed => "Bootstrap key has already been used.",
        }
    }
}

impl Display for AuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let description = self.description();
        write!(f, "{description}")
    }
}

impl std::error::Error for AuthError {}
