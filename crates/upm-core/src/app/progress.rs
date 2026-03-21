#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationKind {
    Add,
    Search,
    UpdateBatch,
    UpdateItem,
    Remove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationStage {
    ResolveQuery,
    DiscoverRelease,
    SelectArtifact,
    DownloadArtifact,
    StagePayload,
    WriteDesktopEntry,
    ExtractIcon,
    RefreshIntegration,
    SaveRegistry,
    Finalize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationEvent {
    Started {
        kind: OperationKind,
        label: String,
    },
    StageChanged {
        stage: OperationStage,
        message: String,
    },
    Progress {
        current: u64,
        total: Option<u64>,
    },
    Warning {
        message: String,
    },
    Finished {
        summary: String,
    },
    Failed {
        stage: OperationStage,
        reason: String,
    },
}

pub trait ProgressReporter {
    fn report(&mut self, event: &OperationEvent);
}

impl<F> ProgressReporter for F
where
    F: FnMut(&OperationEvent),
{
    fn report(&mut self, event: &OperationEvent) {
        self(event);
    }
}

#[derive(Default)]
pub struct NoopReporter;

impl ProgressReporter for NoopReporter {
    fn report(&mut self, _event: &OperationEvent) {}
}
