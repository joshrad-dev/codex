use std::time::Duration;
use std::time::Instant;

use tokio::task::JoinHandle;

use crate::error::CodexErr;
use crate::error::Result as CodexResult;

use super::RegularTask;

pub(crate) struct StartupPrewarmHandle {
    task: JoinHandle<CodexResult<RegularTask>>,
    started_at: Instant,
}

impl StartupPrewarmHandle {
    pub(crate) fn new(task: JoinHandle<CodexResult<RegularTask>>, started_at: Instant) -> Self {
        Self { task, started_at }
    }

    pub(crate) async fn resolve(self) -> (StartupPrewarmResolution, Duration) {
        let Self { task, started_at } = self;
        let age_at_first_turn = started_at.elapsed();

        if !task.is_finished() {
            task.abort();
            return (StartupPrewarmResolution::AbortedNotReady, age_at_first_turn);
        }

        let resolution = match task.await {
            Ok(Ok(regular_task)) => StartupPrewarmResolution::Ready(Box::new(regular_task)),
            Ok(Err(err)) => StartupPrewarmResolution::Failed(err),
            Err(err) => StartupPrewarmResolution::JoinFailed(err),
        };

        (resolution, age_at_first_turn)
    }
}

pub(crate) enum StartupPrewarmResolution {
    Ready(Box<RegularTask>),
    AbortedNotReady,
    Failed(CodexErr),
    JoinFailed(tokio::task::JoinError),
}

impl StartupPrewarmResolution {
    pub(crate) fn metric_status(&self) -> &'static str {
        match self {
            Self::Ready(_) => "consumed",
            Self::AbortedNotReady => "aborted_not_ready",
            Self::Failed(_) => "failed",
            Self::JoinFailed(_) => "join_failed",
        }
    }
}
