use std::path::Path;

use crate::framework::ReportProgress;
use crate::lua::draw_runtime::DrawRuntime;
use crate::prelude::*;
use crate::puzzles::test_algo;

type ThreadFunc = Box<dyn (Fn(&Box<dyn ReportProgress>) -> String) + Send>;
pub struct Algorithm {
    pub draw_runtime: DrawRuntime,
    pub thread_func: ThreadFunc,
}

pub fn load(day: &str, part: &str) -> Result<Algorithm> {
    let thread_func: ThreadFunc = match (day, part) {
        ("test_algo", "part_one") => Box::new(|_| test_algo::part_one().to_string()),
        ("test_algo", "part_two") => Box::new(|rp| test_algo::part_two(rp).to_string()),
        (_, _) => bail!("Couldn't find {day} {part}"),
    };

    let file_path = Path::new("scripts/puzzles")
        .join(day)
        .join(format!("{part}.lua"));

    let draw_runtime = DrawRuntime::new(&file_path);

    Ok(Algorithm {
        draw_runtime,
        thread_func,
    })
}
