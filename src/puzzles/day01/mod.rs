// Day 1: Calorie Counting

use std::str::FromStr;

use itertools::Itertools;

use crate::prelude::*;
#[derive(Debug, Clone, PartialEq, Eq)]
struct ElfInventory(Vec<i64>);
impl From<Vec<i64>> for ElfInventory {
    fn from(input: Vec<i64>) -> Self {
        Self(input)
    }
}

fn parse_input(input: &str) -> Result<Vec<ElfInventory>> {
    let lines = input.lines().collect_vec();
    let groups = lines
        .split(|it| it.trim().is_empty())
        .filter(|group| !group.is_empty())
        .collect_vec();
    groups
        .into_iter()
        .map(|group| {
            group
                .iter()
                .map(|number| i64::from_str(number).map_err(|it| it.into()))
                .collect::<Result<Vec<i64>>>()
        })
        .map_ok(|it| it.into())
        .collect()
}

fn find_max_inventory(inventories: &[ElfInventory]) -> Option<i64> {
    let calories: Vec<i64> = inventories.iter().map(|elf| elf.0.iter().sum()).collect();
    calories.into_iter().max()
}

fn find_top_three(inventories: &[ElfInventory]) -> Vec<i64> {
    let mut calories_per_elf: Vec<i64> = inventories.iter().map(|elf| elf.0.iter().sum()).collect();
    calories_per_elf.sort_unstable();
    calories_per_elf.reverse();
    calories_per_elf.into_iter().take(3).collect_vec()
}

pub fn part_one() -> Result<i64> {
    let inventories = parse_input(include_str!("./puzzle_input.txt"))?;
    let result = find_max_inventory(&inventories);
    result.ok_or_else(|| anyhow!("No result found"))
}

pub fn part_two() -> Result<i64> {
    let inventories = parse_input(include_str!("./puzzle_input.txt"))?;
    let result: i64 = find_top_three(&inventories).iter().sum();
    return Ok(result);
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use super::*;

    const SAMPLE_INPUT: &str = indoc! {"
        1000
        2000
        3000

        4000

        5000
        6000

        7000
        8000
        9000

        10000
    "};

    #[test]
    fn max_inventory() {
        let inventories = parse_input(SAMPLE_INPUT).unwrap();
        let result = find_max_inventory(&inventories);
        assert_eq!(result, Some(24000));
    }

    #[test]
    fn part_one_answer() {
        let result = part_one();
        assert_eq!(result.unwrap(), 72511);
    }

    #[test]
    fn top_three() {
        let inventories = parse_input(SAMPLE_INPUT).unwrap();
        let top_three = find_top_three(&inventories);
        assert_eq!(top_three, vec![24000, 11000, 10000]);
    }
}
