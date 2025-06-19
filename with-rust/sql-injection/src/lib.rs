use clap::Parser;
use reqwest::blocking::{Client, ClientBuilder};
use std::{sync::LazyLock, time::Duration};
use url::Url;

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
