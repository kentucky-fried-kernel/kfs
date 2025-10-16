# Builds the tests, parses Cargo's JSON output, and writes the absolute path to the
# test executable to stdout.
# Example Usage:
# export TEST_EXECUTABLE_PATH=$(python3 testbuild.py)
import subprocess
import sys
import json

proc = subprocess.run(
    ["cargo", "build", "--tests", "--release", "--lib", "--message-format=json"],
    capture_output=True,
)

if proc.returncode != 0:
    print("Build failed", file=sys.stderr)
    print(proc.stdout.decode(), file=sys.stderr)
    print(proc.stderr.decode(), file=sys.stderr)
    sys.exit(1)

outputs = [
    json.loads(output) for output in proc.stdout.decode().split("\n") if output != ""
]

for output in outputs:
    if not (executable := output.get("executable")):
        continue
    if "target/i386-unknown-none/release/deps/kfs-" not in executable:
        continue
    if output["target"]["kind"][0] != "lib":
        continue

    print(executable)
    break
