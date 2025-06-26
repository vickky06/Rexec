#[derive(Debug, Default)]

pub struct ActivityType {
    pub container: Option<String>,
    pub image: Option<String>,
    pub all_tars: Option<String>,
    pub tar: Option<String>,
    pub ports: Option<Vec<i32>>,
}

pub struct CleanupService;
