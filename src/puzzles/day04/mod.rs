// Day 4: Camp Cleanup

use std::str::FromStr;

use chumsky::prelude::*;
use serde::Serialize;

use crate::{framework::ReportProgress, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct Assignment {
    start: i64,
    end: i64,
}

impl Assignment {
    fn contains(&self, other: &Self, report_progress: &impl ReportProgress) -> bool {
        self.contains_point(other.start, report_progress)
            && self.contains_point(other.end, report_progress)
    }

    fn overlaps_with(&self, other: &Self, report_progress: &impl ReportProgress) -> bool {
        self.contains_point(other.start, report_progress)
            || self.contains_point(other.end, report_progress)
            || other.contains_point(self.start, report_progress)
            || other.contains_point(self.end, report_progress)
    }

    fn contains_point(&self, point: i64, report_progress: &impl ReportProgress) -> bool {
        if point >= self.start && point <= self.end {
            report_progress.report_progress(Box::new(ProgressEvent::IntersectionFound {
                position: point,
            }));
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct Pair(Assignment, Assignment);

impl Pair {
    fn has_overlap(&self, report_progress: &impl ReportProgress) -> bool {
        self.0.overlaps_with(&self.1, report_progress)
    }

    fn has_full_overlap(&self, report_progress: &impl ReportProgress) -> bool {
        if self.0.contains(&self.1, report_progress) {
            report_progress.report_progress(Box::new(ProgressEvent::ContainsOther { which: 1 }));
            true
        } else if self.1.contains(&self.0, report_progress) {
            report_progress.report_progress(Box::new(ProgressEvent::ContainsOther { which: 0 }));
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
struct Input(Vec<Pair>);

impl FromStr for Input {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let assignment = || {
            text::int::<_, Simple<_>>(10)
                .then_ignore(just('-'))
                .then(text::int(10))
                .map(|(start, end)| {
                    let start = start.parse().unwrap();
                    let end = end.parse().unwrap();
                    Assignment { start, end }
                })
        };

        let pair = || {
            assignment()
                .then_ignore(just(','))
                .then(assignment())
                .map(|(left, right)| Pair(left, right))
        };

        let parser = pair()
            .separated_by(text::newline())
            .padded()
            .map(|pairs| Input(pairs));

        parser.parse(s).map_err(|err| {
            err.iter()
                .map(|it| it.to_string())
                .join("\n")
                .pipe(|it| anyhow!(it))
        })
    }
}

#[derive(Debug, Serialize)]
enum ProgressEvent {
    AnalyzePair(Pair),
    IntersectionFound { position: i64 },
    ContainsOther { which: u8 },
}

pub fn part_one(report_progress: &impl ReportProgress) -> Result<usize> {
    let input: Input = include_str!("./puzzle_input.txt").parse()?;
    let overlaps = input
        .0
        .iter()
        .filter(|pair| {
            report_progress.report_progress(Box::new(ProgressEvent::AnalyzePair(**pair)));
            pair.has_full_overlap(report_progress)
        })
        .count();
    Ok(overlaps)
}

pub fn part_two(report_progress: &impl ReportProgress) -> Result<usize> {
    let input: Input = include_str!("./puzzle_input.txt").parse()?;
    let overlaps = input
        .0
        .iter()
        .filter(|pair| {
            report_progress.report_progress(Box::new(ProgressEvent::AnalyzePair(**pair)));
            pair.has_overlap(report_progress)
        })
        .count();
    Ok(overlaps)
}

#[cfg(test)]
mod test {
    use crate::framework::NoOpReportProgress;

    use super::*;

    #[test]
    fn part_one_answer() {
        let report_progress: Box<dyn ReportProgress> = Box::new(NoOpReportProgress);
        assert_eq!(part_one(&report_progress).unwrap(), 305);
    }

    #[test]
    fn part_two_answer() {
        let report_progress: Box<dyn ReportProgress> = Box::new(NoOpReportProgress);
        let result = part_two(&report_progress).unwrap();
        assert_eq!(result, 811);
    }

    #[test]
    fn troublesome_overlap() {
        let report_progress: Box<dyn ReportProgress> = Box::new(NoOpReportProgress);
        let pair = Pair(
            Assignment { start: 6, end: 21 },
            Assignment { start: 5, end: 26 },
        );
        assert!(pair.has_overlap(&report_progress));
    }
}
