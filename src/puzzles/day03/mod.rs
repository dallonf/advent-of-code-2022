// Day 3: Rucksack Reorganization

use std::str::FromStr;
use std::fmt::Debug;

use crate::prelude::*;

#[derive(Clone)]
struct Rucksack(Vec<char>, Vec<char>);

impl Debug for Rucksack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Rucksack").field(&self.0.iter().join("")).field(&self.1.iter().join("")).finish()
    }
}

#[derive(Debug, Clone)]
struct Input(Vec<Rucksack>);

impl FromStr for Input {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rucksacks = s
            .lines()
            .map(|line| {
                let chars = line.chars().collect_vec();
                if chars.len() % 2 == 0 {
                    let (left, right) = chars.split_at(chars.len() / 2);
                    Ok(Rucksack(left.to_vec(), right.to_vec()))
                } else {
                    Err(anyhow!("Each line must have an even number of characters"))
                }
            })
            .collect::<Result<Vec<Rucksack>>>();

        rucksacks.map(|it| Input(it))
    }
}

const SAMPLE_INPUT: &str = indoc! {"
  vJrwpWtwJgWrhcsFMMfFFhFp
  jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
  PmmdzqPrVvPwwTWBwg
  wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
  ttgJtRGJQctTZtZT
  CrZsJsPPZsGzwwsLwLmpwMDw
"};

pub fn part_one() -> Result<i64> {
    let parsed: Input = SAMPLE_INPUT.parse()?;
    dbg!(&parsed);
    todo!()
}

pub fn part_two() -> Result<i64> {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;
}
