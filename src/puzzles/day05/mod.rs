// Day 5: Supply Stacks

use std::{collections::VecDeque, str::FromStr};

use chumsky::prelude::*;

use crate::prelude::*;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
struct Stack(VecDeque<char>);
#[derive(Debug, PartialEq, Eq, Clone)]
struct StackCollection(Vec<Stack>);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Instruction {
    from: usize,
    to: usize,
    quantity: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Input {
    stacks: StackCollection,
    instructions: Vec<Instruction>,
}

impl FromStr for Input {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let digit = || filter(|c: &char| c.is_digit(10)).map(|c| c.to_digit(10).unwrap());
        let inline_whitespace = || just(' ').repeated().ignored();
        let crate_parser = {
            just::<_, _, Simple<_>>('[')
                .ignore_then(filter(|c: &char| c.is_alphabetic()))
                .then_ignore(just(']'))
        };
        let empty_space = just("   ").ignored();
        let optional_crate = crate_parser.map(Some).or(empty_space.map(|_| None));
        let crate_row = optional_crate
            .separated_by(just(' '))
            .at_least(1)
            .then_ignore(inline_whitespace());
        let stack_label = digit()
            .delimited_by(just(' '), just(' '))
            .map(|digit| digit as usize);
        let stack_collection = crate_row
            .then_ignore(text::newline())
            .repeated()
            .then(stack_label.separated_by(just(' ')))
            .map(|(rows, labels): (Vec<Vec<Option<char>>>, Vec<usize>)| {
                let num_stacks = labels.len();
                let mut stacks = (0..num_stacks).map(|_| Stack::default()).collect_vec();
                for row in rows.iter().rev() {
                    for (i, stack) in stacks.iter_mut().enumerate() {
                        let row_item = row.get(i).copied().unwrap_or(None);
                        if let Some(row_item) = row_item {
                            stack.0.push_back(row_item);
                        }
                    }
                }
                StackCollection(stacks)
            });

        let instruction = just("move ")
            .ignore_then(text::int(10))
            .then_ignore(just(" from "))
            .then(digit())
            .then_ignore(just(" to "))
            .then(digit())
            .map(|((a, b), c)| Instruction {
                quantity: usize::from_str(&a).unwrap(),
                from: b as usize,
                to: c as usize,
            });

        let instructions = instruction
            .then_ignore(inline_whitespace())
            .separated_by(text::newline());

        let input = stack_collection
            .then_ignore(
                text::newline()
                    .padded_by(inline_whitespace())
                    .repeated()
                    .exactly(2),
            )
            .then(instructions)
            .map(|(stacks, instructions)| Input {
                stacks,
                instructions,
            });

        input.parse(s).map_err(|errs: Vec<Simple<_>>| {
            anyhow!(errs
                .into_iter()
                .map(|err| {
                    let line = &s[0..err.span().start()]
                        .chars()
                        .filter(|&c| c == '\n')
                        .count();
                    format!("{err} at line {line}")
                })
                .join("\n"))
        })
    }
}

const SAMPLE_INPUT: &str = indoc! {"
      [D]    
  [N] [C]    
  [Z] [M] [P]
   1   2   3 

  move 1 from 2 to 1
  move 3 from 1 to 3
  move 2 from 2 to 1
  move 1 from 1 to 2
"};

pub fn part_one() -> Result<String> {
    dbg!(SAMPLE_INPUT.parse::<Input>());
    todo!()
}

pub fn part_two() -> Result<String> {
    todo!()
}
