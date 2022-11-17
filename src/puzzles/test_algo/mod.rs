// 2019 Day 1: The Tyranny of the Rocket Equation

use crate::framework::ReportProgress;
use crate::prelude::*;
use serde::Serialize;

lazy_static! {
    static ref PUZZLE_INPUT: Vec<i64> = include_str!("./test_input.txt")
        .lines()
        .map(|it| it.parse().unwrap())
        .collect();
}

pub fn fuel_amount(mass: i64) -> i64 {
    (mass / 3) - 2
}

pub fn part_one() -> i64 {
    PUZZLE_INPUT.iter().map(|&num| fuel_amount(num)).sum()
}

#[derive(Serialize)]
pub enum PartTwoProgress {
    NewPart { mass: i64 },
    AdditionalFuel { mass: i64 },
}

pub fn recursive_fuel_amount(mass: i64, progress: &impl ReportProgress) -> i64 {
    let required_fuel = fuel_amount(mass);
    if required_fuel > 0 {
        progress.report_progress(Box::new(PartTwoProgress::AdditionalFuel {
            mass: required_fuel,
        }));
        let additional_fuel = recursive_fuel_amount(required_fuel, progress);
        return required_fuel + additional_fuel;
    } else {
        return 0;
    };
}

pub fn part_two(progress: &impl ReportProgress) -> i64 {
    PUZZLE_INPUT
        .iter()
        .map(|&num| {
            progress.report_progress(Box::new(PartTwoProgress::NewPart { mass: num }));
            recursive_fuel_amount(num, progress)
        })
        .sum()
}

#[cfg(test)]
mod part_one_test {
    use super::*;

    #[test]
    fn mass_of_12() {
        assert_eq!(fuel_amount(12), 2);
    }
    #[test]
    fn mass_of_14() {
        assert_eq!(fuel_amount(14), 2);
    }
    #[test]
    fn mass_of_1969() {
        assert_eq!(fuel_amount(1969), 654);
    }
    #[test]
    fn mass_of_100_756() {
        assert_eq!(fuel_amount(100_756), 33_583);
    }
    #[test]
    fn part_one_test() {
        let result: i64 = part_one();
        assert_eq!(result, 3394106);
    }
}

#[cfg(test)]
mod part_two_test {
    use crate::framework::NoOpReportProgress;

    use super::*;
    #[test]
    fn test_cases() {
        assert_eq!(recursive_fuel_amount(14, &mut NoOpReportProgress), 2);
        assert_eq!(recursive_fuel_amount(1969, &mut NoOpReportProgress), 966);
        assert_eq!(
            recursive_fuel_amount(100756, &mut NoOpReportProgress),
            50346
        );
    }
    #[test]
    fn part_two_test() {
        let result: i64 = part_two(&mut NoOpReportProgress);
        assert_eq!(result, 5088280);
    }
}
