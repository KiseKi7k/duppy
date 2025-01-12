use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs;
use walkdir::WalkDir;

pub fn hashing(data: Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);

    format!("{:x}", hasher.finalize())
}

pub fn find_duplicate_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut result = String::new();
    let mut duplicate_map: HashMap<String, Vec<String>> = HashMap::new();

    let _: usize = search_through(path, &mut duplicate_map).expect("Error in search_through");
    if duplicate_map.len() == 0 {
        result.push_str("Not found any duplicate files");
    } else {
        let duplicate_file = find_first(duplicate_map, path).expect("Error in find_first");
        result = format_result(&duplicate_file);
    }

    println!("{}", result);

    Ok(())
}

pub fn create_pgbar(n: u64, msg: &str) -> ProgressBar {
    let fmsg = Cow::Owned(format!("{} Complete", msg));
    let template = "{msg}\n[{elapsed_precise}] [{bar}] {pos:>5}/{len:10}";
    let pgbar = ProgressBar::new(n)
        .with_message(msg.to_string())
        .with_finish(ProgressFinish::WithMessage(fmsg));

    pgbar.set_style(
        ProgressStyle::default_bar()
            .template(template)
            .unwrap()
            .progress_chars("=>-"),
    );

    pgbar
}

pub fn set_pgbar_pathmsg(pgbar: &ProgressBar, main_msg: &str, path: &str) {
    let new_msg = format!("{}\nReading at: {}", main_msg, path);
    pgbar.set_message(Cow::Owned(new_msg));
}

fn search_through(
    path: &str,
    map: &mut HashMap<String, Vec<String>>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let file_count = WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .count();
    let mut set: HashSet<String> = HashSet::new();

    let pgbar_msg = "Finding Duplicate Files...";
    let pgbar = create_pgbar(file_count.try_into().unwrap(), pgbar_msg);

    for entry in WalkDir::new(path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            set_pgbar_pathmsg(&pgbar, &pgbar_msg, path.display().to_string().as_str());
            let data = fs::read(path)?;
            let hash = hashing(data);

            if set.contains(&hash) {
                //println!("Found duplicate file in: {}", path.display());
                let str_path = path.to_str().unwrap().to_string();
                map.entry(hash).or_insert_with(Vec::new).push(str_path);
            } else {
                set.insert(hash);
            }
            pgbar.inc(1);
        }
    }
    Ok(file_count)
}

pub fn find_first(
    mut map: HashMap<String, Vec<String>>,
    path: &str,
    //file_count: usize,
) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let mut dup_vec: Vec<Vec<String>> = Vec::new();
    let key_count = map.len();

    let pgbar_main = create_pgbar(key_count.try_into().unwrap(), "Finding Original Files...");

    //let pgbar_sub = create_pgbar(file_count.try_into().unwrap(), "Searching through files...");

    for entry in WalkDir::new(path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let data = fs::read(path)?;
            let hash = hashing(data);

            if let Some(vec) = map.get_mut(&hash) {
                let str_path = path.to_str().unwrap().to_string();
                vec.push(str_path);
                dup_vec.push(vec.clone());
                map.remove(&hash);

                pgbar_main.inc(1);

                if map.is_empty() {
                    break;
                };
            }
        }
    }

    Ok(dup_vec)
}

fn format_result(results: &Vec<Vec<String>>) -> String {
    let mut output = String::new();
    for (i, result) in results.iter().enumerate() {
        let header = format!(
            "\n*---------------*\n{}. Found {} files duplicated.",
            i + 1,
            result.len()
        );
        output.push_str(&header);
        for r in result.iter() {
            let child = format!("\n{}", r);
            output.push_str(&child);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    //use super::*;

    //#[test]
    //fn test_find_duplicate() {
    //    assert_eq!(
    //        find_duplicate_file(r"H:\Pics").unwrap()[0],
    //        [
    //            "H:\\Pics\\Elaina\\Elaina-1.jpg",
    //            "H:\\Pics\\Elaina.jpg",
    //            "H:\\Pics\\Elaina\\Another Elaina\\Elaina-Extra.jpg"
    //        ]
    //    )
    //}

    //#[test]
    //fn test_search_through() {
    //    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    //
    //    assert_eq!(search_through(r"H:\Pics", &mut map).unwrap(), 4);
    //}

    //#[test]
    //fn test_find_first() {
    //    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    //
    //    let key = "50349139577c15216422a25f8a95ede618e2a19a67a1b6958529f9ab7deb8527".to_string();
    //    let value = vec![
    //        "H:\\Pics\\Elaina\\Elaina-1.jpg".to_string(),
    //        "H:\\Pics\\Elaina.jpg".to_string(),
    //    ];
    //    map.insert(key, value);

    //    assert_eq!(find_first(map, r"H:\Pics", 2).unwrap()[0].len(), 3)
    //}
}
