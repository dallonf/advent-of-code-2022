// Day 3: Rucksack Reorganization

use std::collections::HashSet;
use std::fmt::Debug;
use std::str::FromStr;

use crate::prelude::*;

#[derive(Clone)]
struct Rucksack(Vec<char>, Vec<char>);

impl Debug for Rucksack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Rucksack")
            .field(&self.0.iter().join(""))
            .field(&self.1.iter().join(""))
            .finish()
    }
}

impl Rucksack {
    fn all_items(&self) -> Vec<char> {
        let mut new_vec = self.0.clone();
        new_vec.extend_from_slice(&self.1);
        new_vec
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

fn get_repeated_item(rucksack: &Rucksack) -> Option<char> {
    let second_compartment_set: HashSet<char> = rucksack.1.iter().copied().collect();
    for c in rucksack.0.iter() {
        if second_compartment_set.contains(&c) {
            return Some(*c);
        }
    }
    return None;
}

fn get_priority(item: char) -> Result<i64> {
    match item {
        'a'..='z' => Ok((item as i64 - 'a' as i64) + 1),
        'A'..='Z' => Ok((item as i64 - 'A' as i64) + 27),
        _ => Err(anyhow!("{item} doesn't have a priority")),
    }
}

fn find_badges(input: &Input) -> Result<i64> {
    let all_badges: Result<Vec<i64>> = input
        .0
        .chunks(3)
        .map(|group| {
            let sets = group
                .iter()
                .map(|rucksack| HashSet::<_>::from_iter(rucksack.all_items().into_iter()));
            let combined_set = sets
                .reduce(|prev, next| prev.intersection(&next).copied().collect())
                .unwrap();
            combined_set
                .iter()
                .exactly_one()
                .map_err(|err| anyhow!(err.to_string()))
                .and_then(|it| get_priority(*it))
        })
        .collect();

    Ok(all_badges?.iter().sum())
}

pub fn part_one() -> Result<i64> {
    let parsed: Input = include_str!("./puzzle_input.txt").parse()?;
    let result: i64 = parsed
        .0
        .iter()
        .map(|it| {
            get_repeated_item(&it)
                .ok_or_else(|| anyhow!("couldn't find a repeated item for {it:?}"))
        })
        .map_ok(|it| get_priority(it))
        .flatten_ok()
        .collect::<Result<Vec<i64>>>()?
        .iter()
        .sum();
    Ok(result)
}

pub fn part_two() -> Result<i64> {
    let parsed: Input = include_str!("./puzzle_input.txt").parse()?;
    find_badges(&parsed)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part_one_answer() {
        assert_eq!(part_one().unwrap(), 8243);
    }

    #[test]
    fn part_two_answer() {
        assert_eq!(part_two().unwrap(), 2631);
    }
}
