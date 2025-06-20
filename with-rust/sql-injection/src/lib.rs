use clap::Parser;
use regex::Regex;
use reqwest::blocking::{Client, ClientBuilder};
use scraper::{Html, Selector};
use std::{sync::LazyLock, time::Duration};
use url::Url;

pub static LOGIN_CSRF_TOKEN_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("form input[name=csrf]").unwrap());

pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .cookie_store(true)
        .build()
        .expect("[-] Failed to generate reqwest blocking client")
});

pub static LAB_IS_SOLVED_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Congratulations, you solved the lab!")
        .expect("[-] Failed to generate LAB_IS_SOLVED_REGEX")
});

#[derive(Parser, Debug)]
pub struct Args {
    /// Your Lab Instance URL is Required
    #[arg(short, long, value_parser = parse_url)]
    pub lab_url: Url,
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
