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
    #[derive(Default)]
    struct Crawler {
        size_cache: HashMap<Vec<String>, u64>,
        matching_dir_sizes: Vec<u64>,
    }
    impl Crawler {
        fn crawl(&mut self, dir: &Directory, path: &[String]) {
            let size = dir.get_size_with_cache(&path, &mut self.size_cache);
            if size <= 100000 {
                self.matching_dir_sizes.push(size);
            }
            for (name, child) in dir.items.iter().filter_map(|(name, child)| match child {
                Item::Directory(dir) => Some((name, dir)),
                Item::File { .. } => None,
            }) {
                let mut new_path = path.to_vec();
                new_path.push(name.to_owned());
                self.crawl(child, &new_path);
            }
        }
    }
    let mut crawler = Crawler::default();
    crawler.crawl(fs, &vec![]);
    crawler.matching_dir_sizes.iter().sum()
}

pub fn part_one() -> Result<u64> {
    let fs = Directory::fs_from_input(include_str!("./puzzle_input.txt"))?;
    Ok(crawl_for_small_dirs(&fs))
}

pub fn part_two() -> Result<u64> {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part_one_answer() {
        assert_eq!(part_one().unwrap(), 1077191);
    }
}
