use framework::DrawContext;
use ggez::GameError;

mod test_algo;

#[no_mangle]
pub fn run(day: &str, part: &str, ctx: &'static DrawContext) -> Result<String, GameError> {
    match (day, part) {
        ("test_algo", "part_two") => {
            test_algo::part_two_viz(ctx).map_err(|err| GameError::CustomError(err.to_string()))
        }
        (_, _) => Err(GameError::CustomError(format!(
            "Couldn't run ${day} ${part}"
        ))),
    }
}
