use std::path::Path;

use crate::framework::ReportProgress;
use crate::lua::draw_runtime::DrawRuntime;
use crate::prelude::*;
use crate::puzzles;

pub type ThreadFunc = Box<dyn (Fn(&Box<dyn ReportProgress>) -> Result<String>) + Send>;
pub struct Algorithm {
    pub draw_runtime: DrawRuntime,
    pub thread_func: ThreadFunc,
}

pub fn load(day: &str, part: &str) -> Result<Algorithm> {
    let thread_func: ThreadFunc = match (day, part) {
        ("test_algo", "part_one") => Box::new(|_| Ok(puzzles::test_algo::part_one().to_string())),
        ("test_algo", "part_two") => {
            Box::new(|rp| Ok(puzzles::test_algo::part_two(rp).to_string()))
        }
        ("day01", "part_one") => Box::new(|_| puzzles::day01::part_one().map(|it| it.to_string())),
        ("day01", "part_two") => Box::new(|_| puzzles::day01::part_two().map(|it| it.to_string())),
        ("day02", "part_one") => Box::new(|_| puzzles::day02::part_one().map(|it| it.to_string())),
        ("day02", "part_two") => Box::new(|_| puzzles::day02::part_two().map(|it| it.to_string())),
        ("day03", "part_one") => Box::new(|_| puzzles::day03::part_one().map(|it| it.to_string())),
        ("day03", "part_two") => Box::new(|_| puzzles::day03::part_two().map(|it| it.to_string())),
        ("day04", "part_one") => {
            Box::new(|progress| puzzles::day04::part_one(progress).map(|it| it.to_string()))
        }
        ("day04", "part_two") => Box::new(|_| puzzles::day04::part_two().map(|it| it.to_string())),
        (_, _) => bail!("Couldn't find {day} {part}"),
    };

    let file_path = Path::new("scripts")
        .join("puzzles")
        .join(day)
        .join(format!("{part}.lua"));

    let draw_runtime = DrawRuntime::new(&file_path);

    Ok(Algorithm {
        draw_runtime,
        thread_func,
    })
}
