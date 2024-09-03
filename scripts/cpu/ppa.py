#!/usr/bin/env python3

from pathlib import Path
import subprocess
import json

from constants import *
from utils import *


def run_openroad():
    if not os.path.exists(openroad_flow):
        print(f"ERROR: `OPENROAD_FLOW` environment variable is not a path.")
        exit(1)

    # Run OpenRoad
    subprocess.run(
        [
            "make",
            "-C",
            openroad_flow,
            f"DESIGN_CONFIG={cpu_script_dir}/openroad/nangate45/config.mk",
            f"WORK_HOME={cpu_script_dir}/openroad"
        ],
        cwd=openroad_flow,
    )


def report():
    report_path = Path(cpu_script_dir) / Path("openroad/logs/nangate45/Core/base/6_report.json")
    if not os.path.exists(report_path):
        print(f"ERROR: {report_path} doesn't exist")
        exit(1)

    # Parse power and area.
    with open(report_path, 'r', encoding='utf-8') as report_file:
        report = json.load(report_file)
        total_negative_slack = report["finish__timing__setup__tns"]
        total_power = report["finish__power__total"]
        design_instance_area = report["finish__design__instance__area__stdcell"]

    logger.info(f"=== Results ===")
    logger.info(f"Total Negative Slack: {total_negative_slack}")
    logger.info(f"Total Power: {total_power}")
    logger.info(f"Design Area: {design_instance_area}")
    logger.info(f"Visit {report_path} for details")


if __name__ == "__main__":
    run_openroad()
    report()
