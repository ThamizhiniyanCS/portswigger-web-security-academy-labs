use clap::Parser;
use url::Url;

#[derive(Parser, Debug)]
#[command(name = "Remote Code Execution Via Webshell Upload")]
#[command(bin_name = "01-remote-code-execution-via-webshell-upload")]
#[command(version, about, long_about = None)]
struct Args {
    /// Your Lab Instance URL is Required
    #[arg(short, long, value_parser = parse_url)]
    lab_url: Url,

    /// Choose the lab number
    #[arg(short, long)]
    lab_number: u8,
}

fn parse_url(s: &str) -> Result<Url, String> {
    Url::parse(s).map_err(|e| format!("[-] Invalid URL: {}", e))
}

fn main() {
    let args = Args::parse();
    let lab_url = args.lab_url.as_str().trim_end_matches("/");

    println!("{lab_url}")
}
