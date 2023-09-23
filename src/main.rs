use std::io::Write;

use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};

const GITIGNORE_REPOSITORY_BASE_URL: &str =
    "https://api.github.com/repos/github/gitignore/contents";

#[derive(Serialize, Deserialize)]
struct ResponseDTO {
    name: String,
    path: String,
    sha: String,
    size: usize,
    url: String,
    html_url: String,
    git_url: String,
    download_url: String,
    content: String,
    encoding: String,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 || args.len() > 2 {
        print_usage();
        std::process::exit(1);
    }

    let template = match args.iter().nth(1) {
        Some(s) => s,
        None => {
            print_usage();
            std::process::exit(1)
        }
    };

    let res_body = fetch_gitignore_contents(&template).expect("Failed to fetch gitignore template");
    let response_dto: ResponseDTO =
        serde_json::from_str(&res_body).expect("Failed to deserialize response from github");

    let gitignore_content = decode_contents(&remove_whitespace(&response_dto.content))
        .expect("Failed to decode gitignore content");

    let cwd = std::env::current_dir()
        .expect("Failed to read current directory")
        .to_str()
        .expect("Failed to read current directory")
        .to_string();

    let overwrite = if file_exists(format!("{cwd}/.gitignore")) {
        eprintln!("Overwrite existing gitignore? [y]es / [n]o");
        let ans = get_answer().to_lowercase().trim().to_string();
        ans
    } else {
        "y".to_string()
    };

    if overwrite != "y" {
        eprintln!("Aborting...");
        std::process::exit(0)
    };

    let mut file = std::fs::OpenOptions::new()
        .append(false)
        .truncate(true)
        .write(true)
        .create(true)
        .open(format!("{cwd}/.gitignore"))
        .expect("Failed to open .gitignore");

    file.write_all(format!("{}", &gitignore_content).as_bytes())
        .expect("Failed to write gitignore contents to ./.gitignore");

    eprintln!("gitignore created at: {cwd}/.gitignore");
}

fn fetch_gitignore_contents(template: &String) -> Result<String> {
    let url = format!("{}/{}.gitignore", GITIGNORE_REPOSITORY_BASE_URL, template);
    let res_body = reqwest::blocking::Client::new()
    .get(url)
    .header(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
    .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
    .send()?.text()?;
    Ok(res_body)
}

fn decode_contents(base64_encoded_contents: &String) -> Result<String> {
    let decoded = general_purpose::STANDARD.decode(remove_whitespace(base64_encoded_contents))?;
    Ok(String::from_utf8(decoded.to_vec())?)
}

fn remove_whitespace(s: &String) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn file_exists(path: String) -> bool {
    match std::fs::metadata(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn print_usage() {
    eprintln!("Usage: touch-gitignore <template>");
}

fn get_answer() -> String {
    let mut ans = String::new();
    std::io::stdin()
        .read_line(&mut ans)
        .expect("Failed to read answer");
    ans
}
