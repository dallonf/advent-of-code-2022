use std::path::Path;

use crate::framework::ReportProgress;
use crate::lua::draw_runtime::DrawRuntime;
use crate::prelude::*;
use crate::puzzles::*;

pub type ThreadFunc = Box<dyn (Fn(&Box<dyn ReportProgress>) -> Result<String>) + Send>;
pub struct Algorithm {
    pub draw_runtime: DrawRuntime,
    pub thread_func: ThreadFunc,
}

pub fn load(day: &str, part: &str) -> Result<Algorithm> {
    let thread_func: ThreadFunc = match (day, part) {
        ("test_algo", "part_one") => Box::new(|_| Ok(test_algo::part_one().to_string())),
        ("test_algo", "part_two") => Box::new(|rp| Ok(test_algo::part_two(rp).to_string())),
        ("day01", "part_one") => Box::new(|_| day01::part_one().map(|it| it.to_string())),
        ("day01", "part_two") => Box::new(|_| day01::part_two().map(|it| it.to_string())),
        ("day02", "part_one") => Box::new(|_| day02::part_one().map(|it| it.to_string())),
        ("day02", "part_two") => Box::new(|_| day02::part_two().map(|it| it.to_string())),
        ("day03", "part_one") => Box::new(|_| day03::part_one().map(|it| it.to_string())),
        ("day03", "part_two") => Box::new(|_| day03::part_two().map(|it| it.to_string())),
        ("day04", "part_one") => {
            Box::new(|progress| day04::part_one(progress).map(|it| it.to_string()))
        }
        ("day04", "part_two") => {
            Box::new(|progress| day04::part_two(progress).map(|it| it.to_string()))
        }
        ("day05", "part_one") => Box::new(|_| day05::part_one()),
        ("day05", "part_two") => Box::new(|_| day05::part_two()),
        ("day06", "part_one") => Box::new(|_| day06::part_one().map(|it| it.to_string())),
        ("day06", "part_two") => Box::new(|_| day06::part_two().map(|it| it.to_string())),
        ("day07", "part_one") => Box::new(|_| day07::part_one().map(|it| it.to_string())),
        ("day07", "part_two") => Box::new(|_| day07::part_two().map(|it| it.to_string())),
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
