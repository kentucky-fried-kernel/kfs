import subprocess
import sys

proc = subprocess.run(
    ["cargo", "build", "--tests", "--release", "--message-format=json"],
    capture_output=True,
)

if proc.returncode != 0:
    print(str(proc.stdout), file=sys.stderr)
    print(str(proc.stderr), file=sys.stderr)
    sys.exit(1)

stdout = str(proc.stdout)
haystack = "target/i386-unknown-none/release/deps/kfs-"
index = stdout.find(haystack)
path = stdout[index : index + len(haystack) + 16]
print(path)
