pub struct ErrorCode;

impl ErrorCode {
    pub const GIT_CLONE_FAILED: &'static str = "1001";
    pub const GIT_PULL_FAILED: &'static str = "1002";
    pub const GIT_GET_LAST_COMMIT_TIMESTAMP_FAILED: &'static str = "1003";

    pub const COMPOSE_UP_FAILED: &'static str = "2001";
    pub const COMPOSE_DOWN_FAILED: &'static str = "2002";
    pub const COMPOSE_LIST_CONTAINERS_FAILED: &'static str = "2003";
    pub const COMPOSE_FILE_NOT_FOUND: &'static str = "2004";

    pub const PROJECT_CREATE_FAILED: &'static str = "3001";
    pub const PROJECT_LIST_FAILED: &'static str = "3002";
    pub const PROJECT_INVALID_PATH: &'static str = "3003";
    pub const PROJECT_FILE_READ_FAILED: &'static str = "3004";
    pub const PROJECT_FILE_PARSE_FAILED: &'static str = "3005";
    pub const PROJECT_NOT_FOUND: &'static str = "3006";
}
