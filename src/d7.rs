use itertools::Itertools;
use std::{collections::HashMap, path::PathBuf};

fn parse_files_from_navigation(navigations: &str) -> HashMap<PathBuf, u32> {
    let mut current_path = PathBuf::new();
    let mut files_with_sizes: HashMap<PathBuf, u32> = HashMap::new();

    for input_and_output in navigations.split("\n$ ") {
        if let Some(("cd", dir_name)) = input_and_output.split_once(' ') {
            match dir_name {
                ".." => current_path = current_path.parent().unwrap().to_path_buf(),
                _ => current_path.push(dir_name),
            }
        } else {
            files_with_sizes.extend(
                input_and_output
                    .lines()
                    // skip the input (the call to ls)
                    .skip(1)
                    .filter_map(|line| match line.split_once(' ').unwrap() {
                        ("dir", _dir_name) => None,
                        (file_size, file_name) => {
                            Some((current_path.join(file_name), file_size.parse().unwrap()))
                        }
                    }),
            );
        }
    }
    files_with_sizes
}

fn get_dir_sizes(files_with_sizes: HashMap<PathBuf, u32>) -> HashMap<PathBuf, u32> {
    files_with_sizes
        .into_iter()
        .flat_map(|(file_path, file_size)| {
            file_path
                .ancestors()
                .skip(1)
                .map(|ancestor_path| (ancestor_path.to_path_buf(), file_size))
                .collect_vec()
        })
        .into_grouping_map()
        .sum()
}

pub fn p1(file: &str) -> u32 {
    let upper_bound = 100_000u32;

    let navigations = &file[2..];
    let files_with_sizes = parse_files_from_navigation(navigations);

    let dirs_with_sizes = get_dir_sizes(files_with_sizes);

    dirs_with_sizes
        .values()
        .filter(|dir_size| **dir_size <= upper_bound)
        .sum()
}

pub fn p2(file: &str) -> u32 {
    let navigations = &file[2..];
    let files_with_sizes = parse_files_from_navigation(navigations);

    let dirs_with_sizes = get_dir_sizes(files_with_sizes);

    let total_space = 70_000_000u32;
    let total_used_space = *dirs_with_sizes.get(&PathBuf::from("/")).unwrap();
    let total_available_space = total_space - total_used_space;
    let total_to_free_up = 30_000_000u32;
    let left_to_free_up = total_to_free_up - total_available_space;

    *dirs_with_sizes
        .values()
        .filter(|dir_size| **dir_size >= left_to_free_up)
        .sorted_unstable()
        .next()
        .unwrap()
}
