// Day 2: Rock Paper Scissors

use std::str::FromStr;

use chumsky::prelude::*;
use serde::Serialize;

use crate::prelude::*;

#[derive(Debug, Serialize, Clone)]
struct Entry(char, char);
#[derive(Debug, Serialize, Clone)]
struct Strategy(Vec<Entry>);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum RPSMove {
    Rock,
    Paper,
    Scissors,
}

impl RPSMove {
    fn beats(&self) -> Self {
        match self {
            Self::Rock => Self::Scissors,
            Self::Paper => Self::Rock,
            Self::Scissors => Self::Paper,
        }
    }

    fn beaten_by(&self) -> Self {
        match self {
            Self::Rock => Self::Paper,
            Self::Paper => Self::Scissors,
            Self::Scissors => Self::Rock,
        }
    }
}

impl FromStr for Strategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().collect_vec();
        let char_parser = || filter::<_, _, Simple<char>>(|c: &char| c.is_alphabetic());
        let parser = char_parser()
            .then_ignore(just(' '))
            .then(char_parser())
            .map(|pair| Entry(pair.0, pair.1))
            .then_ignore(end());

        let entries: Result<Vec<Entry>> = lines
            .iter()
            .map(|line| {
                parser
                    .parse(*line)
                    .map_err(|e| anyhow!(e.first().unwrap().clone()))
            })
            .collect();

        Ok(Strategy(entries?))
    }
}

fn round_score(your_move: RPSMove, opponent_move: RPSMove) -> i64 {
    let base_score = match your_move {
        RPSMove::Rock => 1,
        RPSMove::Paper => 2,
        RPSMove::Scissors => 3,
    };

    let outcome_score = match get_winner(your_move, opponent_move) {
        Winner::Right => 0,
        Winner::Tie => 3,
        Winner::Left => 6,
    };

    return base_score + outcome_score;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Winner {
    Left,
    Right,
    Tie,
}

fn get_winner(left: RPSMove, right: RPSMove) -> Winner {
    if left == right {
        Winner::Tie
    } else if left.beats() == right {
        Winner::Left
    } else if left.beaten_by() == right {
        Winner::Right
    } else {
        panic!("This shouldn't be possible")
    }
}

fn get_total_score_for_strategy(strategy: &Strategy) -> Result<i64> {
    let outcomes: Result<Vec<i64>> = strategy
        .0
        .iter()
        .map(|entry| {
            let opponent_move = match entry.0 {
                'A' => RPSMove::Rock,
                'B' => RPSMove::Paper,
                'C' => RPSMove::Scissors,
                other => bail!("Unexpected opponent move: {other}"),
            };

            let your_move = match entry.1 {
                'X' => RPSMove::Rock,
                'Y' => RPSMove::Paper,
                'Z' => RPSMove::Scissors,
                other => bail!("Unexpected player move: {other}"),
            };

            Ok(round_score(your_move, opponent_move))
        })
        .collect();

    Ok(outcomes?.into_iter().sum())
}

pub fn part_one() -> Result<i64> {
    let strategy = include_str!("./puzzle_input.txt").parse()?;
    get_total_score_for_strategy(&strategy)
}

fn get_total_score_for_smarter_strategy(strategy: &Strategy) -> Result<i64> {
    let outcomes: Result<Vec<i64>> = strategy
        .0
        .iter()
        .map(|entry| {
            let opponent_move = match entry.0 {
                'A' => RPSMove::Rock,
                'B' => RPSMove::Paper,
                'C' => RPSMove::Scissors,
                other => bail!("Unexpected opponent move: {other}"),
            };

            let your_move = match entry.1 {
                'X' => opponent_move.beats(),
                'Y' => opponent_move,
                'Z' => opponent_move.beaten_by(),
                other => bail!("Unexpected player move: {other}"),
            };

            Ok(round_score(your_move, opponent_move))
        })
        .collect();

    Ok(outcomes?.into_iter().sum())
}

pub fn part_two() -> Result<i64> {
    let strategy = include_str!("./puzzle_input.txt").parse()?;
    get_total_score_for_smarter_strategy(&strategy)
}

#[cfg(test)]
mod test {
    use super::*;

    const SAMPLE_INPUT: &str = indoc! {"
      A Y
      B X
      C Z
    "};

    #[test]
    fn test_parse() {
        let parsed = Strategy::from_str(SAMPLE_INPUT).unwrap();
        insta::assert_yaml_snapshot!(parsed, @r###"
        ---
        - - A
          - Y
        - - B
          - X
        - - C
          - Z
        "###);
    }

    #[test]
    fn part_one_answer() {
        insta::assert_display_snapshot!(part_one().unwrap(), @"15422");
    }

    #[test]
    fn smarter_strategy() {
        let strategy = Strategy::from_str(SAMPLE_INPUT).unwrap();
        let result = get_total_score_for_smarter_strategy(&strategy).unwrap();
        assert_eq!(result, 12);
    }

    #[test]
    fn part_two_answer() {
        assert_eq!(part_two().unwrap(), 15442);
    }
}
