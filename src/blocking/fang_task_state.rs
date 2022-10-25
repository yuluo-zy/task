/// Possible states of the task
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FangTaskState {
    /// The task is ready to be executed
    New,
    /// The task is being executing.
    ///
    /// The task may stay in this state forever
    /// if an unexpected error happened
    InProgress,
    /// The task failed
    Failed,
    /// The task finished successfully
    Finished,
    /// The task is being retried. It means it failed but it's scheduled to be executed again
    Retried,
}

impl From<String> for FangTaskState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "New" => {
                FangTaskState::New
            }
            "InProgress" => {
                FangTaskState::InProgress
            }
            "Failed" => {
                FangTaskState::Failed
            }
            "Finished" => {
                FangTaskState::Finished
            }
            "Retried" => {
                FangTaskState::Retried
            }
            _ => { FangTaskState::New}
        }
    }
}

impl From<FangTaskState> for String {
    fn from(value: FangTaskState) -> Self {
        match value {
            FangTaskState::New => { String::from("New") }
            FangTaskState::InProgress => { String::from("InProgress") }
            FangTaskState::Failed => { String::from("Failed") }
            FangTaskState::Finished => { String::from("Finished") }
            FangTaskState::Retried => { String::from("Retried") }
        }
    }
}
