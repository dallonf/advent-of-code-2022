// Day 2: Rock Paper Scissors

use std::str::FromStr;

use chumsky::prelude::*;
use serde::Serialize;

use crate::prelude::*;

#[derive(Debug, Serialize, Clone)]
struct Entry(char, char);
#[derive(Debug, Serialize, Clone)]
struct Strategy(Vec<Entry>);

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

pub fn part_one() -> Result<i64> {
    todo!()
}

pub fn part_two() -> Result<i64> {
    todo!()
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
}
