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
    fn overlaps_with(&self, other: &Self, report_progress: &impl ReportProgress) -> bool {
        if self.inside(other.start) {
            report_progress.report_progress(Box::new(ProgressEvent::IntersectionFound {
                position: other.start,
            }));
            return true;
        }
        if self.inside(other.end) {
            report_progress.report_progress(Box::new(ProgressEvent::IntersectionFound {
                position: other.end,
            }));
            return true;
        }
        false
    }

    fn inside(&self, point: i64) -> bool {
        point >= self.start && point <= self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct Pair(Assignment, Assignment);

impl Pair {
    fn has_overlap(&self, report_progress: &impl ReportProgress) -> bool {
        self.0.overlaps_with(&self.1, report_progress)
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

const SAMPLE_INPUT: &str = indoc! {"
  2-4,6-8
  2-3,4-5
  5-7,7-9
  2-8,3-7
  6-6,4-6
  2-6,4-8
"};

#[derive(Debug, Serialize)]
enum ProgressEvent {
    AnalyzePair(Pair),
    IntersectionFound { position: i64 },
}

pub fn part_one(report_progress: &impl ReportProgress) -> Result<usize> {
    let input: Input = SAMPLE_INPUT.parse()?;
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

pub fn part_two() -> Result<i64> {
    todo!()
}
