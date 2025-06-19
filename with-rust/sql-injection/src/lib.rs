use clap::Parser;
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

#[derive(Parser, Debug)]
pub struct Args {
    /// Your Lab Instance URL is Required
    #[arg(short, long, value_parser = parse_url)]
    pub lab_url: Url,
}

fn parse_url(s: &str) -> Result<Url, String> {
    Url::parse(s).map_err(|e| format!("[-] Invalid URL: {}", e))
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
        .expect("[-] CSRF Token Not Found")
}
