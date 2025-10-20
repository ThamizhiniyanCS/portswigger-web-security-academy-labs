use std::sync::LazyLock;

use clap::Parser;
use reqwest::blocking::{Client, ClientBuilder};
use url::Url;

pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    ClientBuilder::new()
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
    Url::parse(s).map_err(|e| logger::error_return(format!("[-] Invalid URL: {}", e).as_ref()))
}

pub fn generate_clap_parser() -> Args {
    Args::parse()
}
