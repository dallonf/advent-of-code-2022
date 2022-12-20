use crate::prelude::*;

pub fn part_one() -> Result<usize> {
    first_new_packet_marker(include_str!("./puzzle_input.txt"))
        .ok_or_else(|| anyhow!("No start-of-packet marker detected."))
}

pub fn part_two() -> Result<String> {
    todo!()
}

const MARKER_LENGTH: usize = 4;
fn first_new_packet_marker(input: &str) -> Option<usize> {
    let char_list: Vec<char> = input.chars().collect();
    for (i, slice) in char_list.windows(MARKER_LENGTH).enumerate() {
        if is_new_packet_marker(slice) {
            return Some(i + MARKER_LENGTH);
        }
    }
    None
}

fn is_new_packet_marker(slice: &[char]) -> bool {
    slice.iter().duplicates().next().is_none()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part_one_examples() {
        assert_eq!(
            first_new_packet_marker("mjqjpqmgbljsphdztnvjfqwrcgsmlb").unwrap(),
            7
        );
        assert_eq!(
            first_new_packet_marker("bvwbjplbgvbhsrlpgdmjqwftvncz").unwrap(),
            5
        );
        assert_eq!(
            first_new_packet_marker("nppdvjthqldpwncqszvftbrmjlhg").unwrap(),
            6
        );
        assert_eq!(
            first_new_packet_marker("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg").unwrap(),
            10
        );
        assert_eq!(
            first_new_packet_marker("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw").unwrap(),
            11
        );
    }

    #[test]
    fn part_one_answer() {
        assert_eq!(part_one().unwrap(), 1093);
    }
}
