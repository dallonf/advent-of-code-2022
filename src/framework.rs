use crate::prelude::*;
use erased_serde::Serialize;
use std::sync::{Arc, Mutex};

pub trait ReportProgress {
    fn report_progress(&self, data: impl Serialize + Send + 'static) -> ();
}

pub struct AsyncReportProgress {
    pub event_bus: EventBus,
}
impl ReportProgress for AsyncReportProgress {
    fn report_progress(&self, data: impl Serialize + Send + 'static) -> () {
        self.event_bus.lock().unwrap().push(Box::new(data))
    }
}

pub struct NoOpReportProgress;
impl ReportProgress for NoOpReportProgress {
    fn report_progress(&self, _data: impl Serialize + 'static) -> () {}
}

pub type Event = dyn Serialize + Send;
pub type EventBus = Arc<Mutex<Vec<Box<Event>>>>;
