#!/usr/bin/env python3

import argparse
import json
import logging
import os
import subprocess
import sys
import typing
from pathlib import Path

LOGGER = logging.getLogger("x")
LOGLEVEL = os.environ.get("LOGLEVEL")
level = logging.DEBUG if LOGLEVEL == "DEBUG" else logging.INFO
LOGGER.setLevel(level)
logging.basicConfig(level=level, format="%(message)s")


def run_with_output(commandline: list[str]) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        commandline,
        stdout=sys.stdout if LOGGER.level <= logging.INFO else subprocess.DEVNULL,
        stderr=sys.stderr if LOGGER.level <= logging.DEBUG else subprocess.DEVNULL,
    )


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Utility to build and run the kernel test suite")
    group = parser.add_argument_group("Test Type").add_mutually_exclusive_group(required=True)
    _ = group.add_argument("--unit-tests", action="store_true")
    _ = group.add_argument("--end-to-end-tests", action="store_true")
    return parser.parse_args(argv[1:])


def build_tests() -> str:
    proc = subprocess.Popen(
        ["cargo", "build", "--tests", "--release", "--lib", "--message-format=json"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    stdout: list[str] = []

    if proc.stdout:
        for line in proc.stdout:
            LOGGER.debug(line)
            stdout.append(line)

    if proc.stderr:
        for line in proc.stderr:
            LOGGER.debug(line)

    exitcode = proc.wait()

    if exitcode != 0:
        raise RuntimeError("Could not build tests")

    return "".join(stdout)


def discover_unit_tests(build_output: str) -> list[str]:
    executables: list[str] = []
    parsed_output: list[dict[str, str | typing.Any]] = [json.loads(o) for o in build_output.split("\n") if o != ""]
    for o in parsed_output:
        if not (exe := o.get("executable")) or "target/i386-unknown-none/release/deps/kfs-" not in exe:
            continue
        if o["target"]["kind"][0] == "bin" or o["target"]["src_path"].endswith("lib.rs"):
            executables.append(exe)

    return executables


def discover_e2e_tests():
    test_names = {str(p.name).strip(".rs") for p in Path("./tests").iterdir() if p.name.endswith(".rs")}
    test_paths = [str(p) for p in Path("./target/i386-unknown-none/release/deps").iterdir() if p.name.split("-")[0] in test_names and not p.name.endswith(".d")]
    return test_paths


def run_tests(type: typing.Literal["E2E", "Unit"]):
    LOGGER.info(f"Building {type} tests...")
    build_output = build_tests()
    test_paths = discover_e2e_tests() if type == "E2E" else discover_unit_tests(build_output)
    LOGGER.info(f"Found {len(test_paths)} {type} test executable(s) to run: {test_paths}")

    ok = 0
    ko = 0
    LOGGER.info(f"Running {type} tests...")

    for path in test_paths:
        LOGGER.info("")
        proc = run_with_output(["./scripts/run.sh", str(path)])
        ok += int(proc.returncode == 0)
        ko += int(proc.returncode != 0)

    LOGGER.info(f"\n{type} test results: {'ok' if ko == 0 else 'FAILED'}. {ok} suite passed; {ko} failed.")


if __name__ == "__main__":
    args = parse_args(sys.argv)
    run_tests("Unit" if args.unit_tests else "E2E")
