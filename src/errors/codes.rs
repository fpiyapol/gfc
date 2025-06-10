pub struct ErrorCode;

impl ErrorCode {
    // Git errors: G1xx
    pub const GIT_CLONE_FAILED: &'static str = "G100";
    pub const GIT_PULL_FAILED: &'static str = "G101";
    pub const GIT_GET_LAST_COMMIT_TIMESTAMP_FAILED: &'static str = "G102";

    // Docker Compose errors: D1xx
    pub const DOCKER_COMPOSE_UP_FAILED: &'static str = "D100";
    pub const DOCKER_COMPOSE_DOWN_FAILED: &'static str = "D101";
    pub const DOCKER_COMPOSE_LIST_CONTAINERS_FAILED: &'static str = "D102";
    pub const DOCKER_COMPOSE_FILE_NOT_FOUND: &'static str = "D103";
}
