from pathlib import Path
import subprocess
import sys
import json
import argparse
import typing


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Utility to build and run the kernel test suite",
    )
    parser.add_argument("--unit-tests", action="store_true")
    parser.add_argument("--end-to-end-tests", action="store_true")
    return parser.parse_args(argv[1:])


def build_tests() -> str:
    """Builds tests, returns stdout on success, throws on failure"""
    proc = subprocess.run(
        ["cargo", "build", "--tests", "--release", "--lib", "--message-format=json"],
        capture_output=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"Could not build tests:\nstdout:\n{proc.stdout.decode()}\nstderr:\n{proc.stderr.decode()}"
        )
    return proc.stdout.decode()


def get_unit_tests_executable_path(build_output: str):
    parsed_output: list[dict[str, typing.Union[str, typing.Any]]] = [
        json.loads(o) for o in build_output.split("\n") if o != ""
    ]
    for o in parsed_output:
        if not (exe := o.get("executable")):
            continue
        if "target/i386-unknown-none/release/deps/kfs-" not in exe:
            continue
        # Not quite sure how reliable this is, we will see if this leads to issues
        # in the future.
        if o["target"]["kind"][0] != "bin":
            continue

        return exe


def run_unit_tests():
    build_output = build_tests()
    test_executable = get_unit_tests_executable_path(build_output)


def discover_e2e_tests():
    test_names = {str(p.name).strip(".rs") for p in Path("./tests").iterdir()}
    return test_names


def run_end_to_end_tests():
    test_paths = discover_e2e_tests()
    print(test_paths)


if __name__ == "__main__":
    args = parse_args(sys.argv)
    if args.unit_tests:
        run_unit_tests()
    else:
        run_end_to_end_tests()
        ...
