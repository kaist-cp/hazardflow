#!/usr/bin/env python3

import os
import argparse
import subprocess

from constants import *


def create_dir(dirname):
    """
    Create directory if `dirname` doesn't exist
    """
    try:
        if not os.path.exists(dirname):
            os.makedirs(dirname)
    except OSError:
        logger.error("Error: Creating directory: " + dirname)


def append_txt_if_keyword_doesnt_exist(file, keyword, txt):
    """
    If `keyword` does not exist in `file`, then write `txt` to `file`
    """
    keyword_exist = False
    with open(file, "r") as f:
        keyword_exist = keyword in f.read()
    if not keyword_exist:
        with open(file, "a") as f:
            f.write(txt.decode("utf-8"))


def check_verilator_version():
    try:
        # Run verilator command to get version information
        result = subprocess.run(
            ["verilator", "--version"], capture_output=True, text=True
        )

        # Check if the command was successful
        if result.returncode == 0:
            # Extract the version number from the output
            version_output = result.stdout.splitlines()[0]  # Extracting the first line
            version_string = version_output.split()[1]  # Assuming version number is the second item

            # Extracting major version number
            major_version = int(version_string.split(".")[0])
            minor_version = int(version_string.split(".")[1][:3])

            return major_version + (minor_version / 1000)
        else:
            print("Error running verilator command:", result.stderr)
            return None
    except FileNotFoundError:
        print("Verilator is not installed or not found in PATH.")
        return None
