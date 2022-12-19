// Day 5: Supply Stacks

use std::{collections::VecDeque, str::FromStr};

use chumsky::prelude::*;

use crate::prelude::*;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
struct Stack(VecDeque<char>);
#[derive(Debug, PartialEq, Eq, Clone)]
struct StackCollection(Vec<Stack>);

impl StackCollection {
    fn follow_instructions_compat(&mut self, instruction: &[Instruction]) {
        for instruction in instruction {
            self.follow_instruction_compat(instruction);
        }
    }

    fn follow_instruction_compat(&mut self, instruction: &Instruction) {
        for _ in 0..instruction.quantity {
            self.move_one_crate(instruction.from, instruction.to);
        }
    }

    fn move_one_crate(&mut self, from: usize, to: usize) {
        let from_stack = &mut self.0[from - 1];
        let crate_to_move = from_stack.0.pop_back();
        if let Some(crate_to_move) = crate_to_move {
            let to_stack = &mut self.0[to - 1];
            to_stack.0.push_back(crate_to_move);
        }
    }

    fn follow_instruction(&mut self, instruction: &Instruction) -> Result<()> {
        let from_stack = &mut self.0[instruction.from - 1];
        let mut moving_stack: Vec<char> = (0..instruction.quantity)
            .map(|_| {
                from_stack
                    .0
                    .pop_back()
                    .ok_or_else(|| anyhow!("No crates to grab"))
            })
            .collect::<Result<_>>()?;
        moving_stack.reverse();
        let to_stack = &mut self.0[instruction.to - 1];
        for crate_to_move in moving_stack {
            to_stack.0.push_back(crate_to_move);
        }
        Ok(())
    }

    fn follow_instructions(&mut self, instructions: &[Instruction]) -> Result<()> {
        for instruction in instructions {
            self.follow_instruction(instruction)?;
        }
        Ok(())
    }
}

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

impl Input {
    fn crates_after_instructions_compat(mut self) -> String {
        self.stacks.follow_instructions_compat(&self.instructions);
        self.top_crates()
    }

    fn top_crates(&self) -> String {
        let top_crates = self
            .stacks
            .0
            .iter()
            .filter_map(|stack| stack.0.back().map(|c| c.to_string()))
            .collect_vec();
        top_crates.join("")
    }

    fn crates_after_instructions(mut self) -> Result<String> {
        self.stacks.follow_instructions(&self.instructions)?;
        Ok(self.top_crates())
    }
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

pub fn part_one() -> Result<String> {
    let input = include_str!("./puzzle_input.txt").parse::<Input>()?;
    Ok(input.crates_after_instructions_compat())
}

pub fn part_two() -> Result<String> {
    let input = include_str!("./puzzle_input.txt").parse::<Input>()?;
    input.crates_after_instructions()
}

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn sample_crates_after_instructions() {
        let input = SAMPLE_INPUT.parse::<Input>().unwrap();
        assert_eq!(&input.crates_after_instructions_compat(), "CMZ");
    }

    #[test]
    fn part_one_answer() {
        assert_eq!(part_one().unwrap(), "FWNSHLDNZ");
    }

    #[test]
    fn sample_crates_after_part_two_instructions() {
        let input = SAMPLE_INPUT.parse::<Input>().unwrap();
        assert_eq!(&input.crates_after_instructions().unwrap(), "MCD");
    }

    #[test]
    fn part_two_answer() {
        assert_eq!(part_two().unwrap(), "RNRGDNFQG");
    }
}
