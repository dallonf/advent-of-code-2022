use erased_serde::Serialize;
use std::sync::mpsc::Sender;

pub trait ReportProgress {
    fn report_progress(&self, data: Box<dyn Serialize + Send>) -> ();
}

impl ReportProgress for Box<dyn ReportProgress> {
    fn report_progress(&self, data: Box<dyn Serialize + Send>) -> () {
        self.as_ref().report_progress(data)
    }
}

pub struct AsyncReportProgress {
    pub sender: Sender<Box<Event>>,
}
impl ReportProgress for AsyncReportProgress {
    fn report_progress(&self, data: Box<dyn Serialize + Send>) -> () {
        self.sender.send(Box::new(data)).unwrap();
    }
}

pub struct NoOpReportProgress;
impl ReportProgress for NoOpReportProgress {
    fn report_progress(&self, _data: Box<dyn Serialize + Send>) -> () {}
}

pub type Event = dyn Serialize + Send;
