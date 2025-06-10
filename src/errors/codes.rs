pub struct ErrorCode;

impl ErrorCode {
    // Git errors: G1xx
    pub const GIT_CLONE_FAILED: &'static str = "G100";
    pub const GIT_PULL_FAILED: &'static str = "G101";
    pub const GIT_GET_LAST_COMMIT_TIMESTAMP_FAILED: &'static str = "G102";
}
