use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs;
use walkdir::WalkDir;

pub struct PGBar {
    progress_bar: ProgressBar,
    msg_template: String,
}

impl PGBar {
    fn create_pgbar(n: u64, msg: &str, final_message: &str) -> ProgressBar {
        let template = "{msg}\n[{elapsed_precise}] [{bar}] {pos:>5}/{len:10}";
        let mut pgbar = ProgressBar::new(n).with_message(msg.to_string());

        if !final_message.is_empty() {
            let final_message = Cow::Owned(final_message.to_string());
            pgbar = pgbar.with_finish(ProgressFinish::WithMessage(final_message));
        }

        pgbar.set_style(
            ProgressStyle::default_bar()
                .template(template)
                .unwrap()
                .progress_chars("=>-"),
        );

        pgbar
    }

    fn create(n: u64, msg_template: &str, final_message: &str) -> Self {
        PGBar {
            progress_bar: Self::create_pgbar(n, msg_template, final_message),
            msg_template: msg_template.to_string(),
        }
    }

    fn update_msg(&self, variables: Vec<&str>) {
        if self.msg_template.matches("{}").count() != variables.len() {
            panic!("Template and varaibles count doesnt match");
        }

        let mut formmatted = self.msg_template.clone();
        for &v in variables.iter() {
            formmatted = formmatted.replacen("{}", v, 1)
        }
        
        self.progress_bar.set_message(formmatted);
    }
}

pub fn run(paths: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let dupe_map = search_and_mapdupe(&paths)?;
    let mut output = String::new();

    if dupe_map.len() > 0 {
        let result = find_first_dupe(&paths, dupe_map)?;
        output = format_result(result);
    } else {
        output.push_str("Not found any duplicate files");
    }

    println!("{}", output);

    Ok(())
}

fn search_and_mapdupe(
    paths: &Vec<String>,
) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    let mut set: HashSet<String> = HashSet::new();
    let mut dupe_map: HashMap<String, Vec<String>> = HashMap::new();

    let pgbar_main_msg_template = "[1/2] ({}/{}) Searching in {}";
    let pgbar_main = PGBar::create(
        paths.len() as u64,
        pgbar_main_msg_template,
        "[1/2] Searching Complete",
    );

    for (i, path) in pgbar_main.progress_bar.wrap_iter(paths.iter().enumerate()) {
        pgbar_main.update_msg(vec![
            &(i+1).to_string(),
            &paths.len().to_string(),
            &path.to_string(),
        ]);
        let file_counts = path_file_counts(&path);

        let pgbar_sub_msg_template = "Reading at: {}";
        let pgbar_sub = PGBar::create(file_counts as u64, pgbar_sub_msg_template, "");

        for entry in pgbar_sub
            .progress_bar
            .wrap_iter(WalkDir::new(path).into_iter())
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                pgbar_sub.update_msg(vec![path.to_str().unwrap()]);
                let data = fs::read(path)?;
                let hash = hashing(data);

                if set.contains(&hash) {
                    let str_path = path.to_str().unwrap().to_string();
                    dupe_map.entry(hash).or_insert_with(Vec::new).push(str_path);
                } else {
                    set.insert(hash);
                }
            }
        }
    }

    assert_eq!(pgbar_main_msg_template, pgbar_main.msg_template);

    Ok(dupe_map)
}

pub fn find_first_dupe(
    paths: &Vec<String>,
    mut dupe_map: HashMap<String, Vec<String>>,
) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let mut dup_vec: Vec<Vec<String>> = Vec::new();
    let key_count = dupe_map.len();

    let pgbar_main_msg_template = "[2/2] ({}/{}) Finding Original Files in {}";
    let pgbar_main = PGBar::create(
        key_count.try_into().unwrap(),
        pgbar_main_msg_template,
        "[2/2] Finding Original Files Complete",
    );

    for (i, path) in pgbar_main.progress_bar.wrap_iter(paths.iter().enumerate()) {
        pgbar_main.update_msg(vec![
            &(i+1).to_string(),
            &paths.len().to_string(),
            &path.to_string(),
        ]);
        let file_counts = path_file_counts(&path);

        let pgbar_sub_msg_template = "Reading at: {}";
        let pgbar_sub = PGBar::create(file_counts as u64, pgbar_sub_msg_template, "");

        for entry in WalkDir::new(path) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                pgbar_sub.update_msg(vec![path.to_str().unwrap()]);
                let data = fs::read(path)?;
                let hash = hashing(data);

                if let Some(vec) = dupe_map.get_mut(&hash) {
                    let str_path = path.to_str().unwrap().to_string();
                    vec.push(str_path);
                    dup_vec.push(vec.clone());
                    dupe_map.remove(&hash);

                    if dupe_map.is_empty() {
                        break;
                    };
                }
            }
        }
    }

    Ok(dup_vec)
}

pub fn hashing(data: Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);

    format!("{:x}", hasher.finalize())
}

fn path_file_counts(path: &str) -> usize {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .count()
}

fn format_result(results: Vec<Vec<String>>) -> String {
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
    use super::PGBar;

    #[test]
    fn test_pgbar() {
        let template = "[{}/{}] ({}/{}) Testing on {} {}";
        let pgbar = PGBar::create(100_u64, template, "");
        let variables = vec!["1", "2", "3", "4", "Hello", "World"];
        pgbar.update_msg(variables);

        assert_eq!(
            pgbar.progress_bar.message(),
            "[1/2] (3/4) Testing on Hello World"
        );
        assert_eq!(pgbar.msg_template, template);
    }

    #[test]
    #[should_panic]
    fn test_pgbar_panic() {
        let template = "{} on {}";
        let pgbar = PGBar::create(100_u64, &template, "End");
        let variables = vec!["Elaina"];

        pgbar.update_msg(variables);
    }
}
