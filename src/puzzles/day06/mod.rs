// Day 6: Tuning Trouble

use crate::prelude::*;

pub fn part_one() -> Result<usize> {
    first_new_packet_marker(include_str!("./puzzle_input.txt"))
        .ok_or_else(|| anyhow!("No start-of-packet marker detected."))
}

pub fn part_two() -> Result<usize> {
    first_unique_slice(include_str!("./puzzle_input.txt"), 14)
        .ok_or_else(|| anyhow!("No start-of-message marker detected."))
}

fn first_new_packet_marker(input: &str) -> Option<usize> {
    first_unique_slice(input, 4)
}

fn first_unique_slice(input: &str, slice_length: usize) -> Option<usize> {
    let char_list: Vec<char> = input.chars().collect();
    let previous_slice_length = slice_length - 1;
    let mut last_duplicate: Option<usize> = None;
    for (i, current_char) in char_list.iter().enumerate() {
        let (slice_start, is_warm) = if i > previous_slice_length {
            (i - previous_slice_length, true)
        } else {
            (0, false)
        };
        let previous_slice = &char_list[slice_start..i];
        for (back_i, back_char) in previous_slice.iter().enumerate().rev() {
            if back_char == current_char {
                let found_duplicate = slice_start + back_i;
                last_duplicate = last_duplicate.map_or(found_duplicate, |prev_duplicate| {
                    found_duplicate.max(prev_duplicate)
                }).pipe(Some);
                break;
            }
        }
        let has_duplicate = last_duplicate.map(|it| it >= slice_start).unwrap_or(false);
        if !has_duplicate && is_warm {
            return Some(i + 1);
        }
    }
    None
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

    #[test]
    fn part_two_answer() {
        assert_eq!(part_two().unwrap(), 3534);
    }
}
