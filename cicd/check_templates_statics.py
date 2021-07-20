#!/usr/bin/env python3
#
# Verifies that all templates and all static files have been integrated into the source code.
#
import glob
import os.path
import sys


TEMPLATE_LINE_SUBSTR = "../templates/"
STATIC_LINE_SUBSTR = "../static/"


def main():
    with open(os.path.join("src", "main.rs")) as f:
        lines = [
            line
            for line in f.readlines()
            if TEMPLATE_LINE_SUBSTR in line
            or STATIC_LINE_SUBSTR in line
        ]

    bad = False

    for template_path in glob.glob(os.path.join("templates", "*.tera")):
        template_name = os.path.basename(template_path)
        template_code_path = f"{TEMPLATE_LINE_SUBSTR}{template_name}"
        if not any(template_code_path in line for line in lines):
            print(f"template {template_name} missing from main.rs")
            bad = True

    for static_path in glob.glob(os.path.join("static", "*")):
        static_name = os.path.basename(static_path)
        static_code_path = f"{STATIC_LINE_SUBSTR}{static_name}"
        if not any(static_code_path in line for line in lines):
            print(f"static file {static_name} missing from main.rs")
            bad = True

    if bad:
        sys.exit(1)


if __name__ == "__main__":
    main()
