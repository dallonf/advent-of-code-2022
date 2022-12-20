// Day 7: No Space Left On Device

use std::collections::HashMap;

use crate::prelude::*;

#[derive(Clone, Debug, Default)]
struct Directory {
    items: HashMap<String, Item>,
}

#[derive(Clone, Debug)]
enum Item {
    Directory(Directory),
    File { size: u64 },
}

impl Directory {
    fn get_size_with_cache(
        &self,
        current_path: &[String],
        cache: &mut HashMap<Vec<String>, u64>,
    ) -> u64 {
        if let Some(cached) = cache.get(current_path) {
            return *cached;
        }
        self.items
            .iter()
            .map(|(path, item)| {
                let mut new_path = current_path.to_vec();
                new_path.push(path.to_owned());
                item.get_size_with_cache(&new_path, cache)
            })
            .sum()
    }

    fn borrow_items_mut(&mut self, path: &[String]) -> Result<&mut HashMap<String, Item>> {
        let mut result = &mut self.items;
        for segment in path {
            let item = result.get_mut(segment);
            match item {
                Some(Item::Directory(dir)) => {
                    result = &mut dir.items;
                }
                Some(Item::File { .. }) => bail!("{segment} is not a directory."),
                None => bail!("{segment} does not exist."),
            }
        }
        Ok(result)
    }

    fn fs_from_input(input: &str) -> Result<Directory> {
        let mut dir = Directory::default();
        let mut current_path: Vec<String> = vec![];
        for command in input.lines() {
            let current_dir_items = dir.borrow_items_mut(&current_path)?;
            if command == "$ cd /" {
                current_path = vec![];
            } else if command == "$ cd .." {
                current_path.pop();
            } else if command.starts_with("$ cd ") {
                let dirname = &command["$ cd ".len()..];
                current_path.push(dirname.to_string());
            } else if command == "$ ls" {
                // no-op
            } else if command.starts_with("dir ") {
                // let
                let dirname = &command["dir ".len()..];
                current_dir_items
                    .insert(dirname.to_string(), Item::Directory(Directory::default()));
            } else if command
                .chars()
                .next()
                .map(|first_char| first_char.is_digit(10))
                .unwrap_or(false)
            {
                let (size, name) = command.split_at(command.find(" ").ok_or_else(|| {
                    anyhow!("expected \" \" separator between filesize and name in \"{command}\"")
                })?);
                let size = u64::from_str_radix(size, 10)?;
                let name = name[1..].to_string(); // split_at() leaves a leading space
                current_dir_items.insert(name, Item::File { size });
            } else {
                bail!("Unrecognized command: \"{command}\"")
            }
        }
        Ok(dir)
    }

    fn dirs_recursive<'a>(
        &'a self,
        path: Vec<String>,
    ) -> Box<dyn Iterator<Item = (Vec<String>, &'a Directory)> + 'a> {
        let path_clone = path.to_vec();
        let children = self
            .items
            .iter()
            .filter_map(move |(name, item)| match item {
                Item::Directory(dir) => {
                    let mut new_path = path_clone.to_vec();
                    new_path.push(name.to_string());
                    Some((new_path, dir))
                }
                Item::File { .. } => None,
            })
            .flat_map(|(path, dir)| dir.dirs_recursive(path));
        Box::new(std::iter::once((path, self)).chain(children))
    }
}

impl Item {
    fn get_size_with_cache(&self, path: &[String], cache: &mut HashMap<Vec<String>, u64>) -> u64 {
        match self {
            Item::Directory(dir) => dir.get_size_with_cache(path, cache),
            Item::File { size } => *size,
        }
    }
}

fn crawl_for_small_dirs(fs: &Directory) -> u64 {
    let mut size_cache = HashMap::<Vec<String>, u64>::new();
    fs.dirs_recursive(vec![])
        .map(|(path, dir)| dir.get_size_with_cache(&path, &mut size_cache))
        .filter(|size| *size <= 100_000)
        .sum()
}

const TOTAL_SPACE: u64 = 70_000_000;
const REQUIRED_SPACE: u64 = 30_000_000;
fn clear_space(fs: &Directory) -> Option<u64> {
    let mut size_cache: HashMap<Vec<String>, u64> = HashMap::new();
    let current_size = fs.get_size_with_cache(&vec![], &mut size_cache);
    let unused_space = TOTAL_SPACE - current_size;
    let amount_to_remove = REQUIRED_SPACE - unused_space;
    fs.dirs_recursive(vec![])
        .map(|(path, dir)| dir.get_size_with_cache(&path, &mut size_cache))
        .filter(|size| *size >= amount_to_remove)
        .min()
}

pub fn part_one() -> Result<u64> {
    let fs = Directory::fs_from_input(include_str!("./puzzle_input.txt"))?;
    Ok(crawl_for_small_dirs(&fs))
}

pub fn part_two() -> Result<u64> {
    let fs = Directory::fs_from_input(include_str!("./puzzle_input.txt"))?;
    clear_space(&fs).ok_or_else(|| anyhow!("Couldn't find a suitable directory"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part_one_answer() {
        assert_eq!(part_one().unwrap(), 1077191);
    }

    #[test]
    fn part_two_answer() {
        assert_eq!(part_two().unwrap(), 5649896);
    }
}
