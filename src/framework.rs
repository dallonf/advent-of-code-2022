use erased_serde::Serialize;
use std::sync::mpsc::{Sender};

pub trait ReportProgress {
    fn report_progress(&self, data: impl Serialize + Send + 'static) -> ();
}

pub struct AsyncReportProgress {
    pub sender: Sender<Box<Event>>,
}
impl ReportProgress for AsyncReportProgress {
    fn report_progress(&self, data: impl Serialize + Send + 'static) -> () {
        self.sender.send(Box::new(data)).unwrap();
    }
}

pub struct NoOpReportProgress;
impl ReportProgress for NoOpReportProgress {
    fn report_progress(&self, _data: impl Serialize + 'static) -> () {}
}

pub type Event = dyn Serialize + Send;
