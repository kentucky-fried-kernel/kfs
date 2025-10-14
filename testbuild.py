import subprocess

proc = subprocess.run(
    ["cargo", "build", "--tests", "--release", "--message-format=json"],
    capture_output=True,
)

stdout = str(proc.stdout)
haystack = "target/i386-unknown-none/release/deps/kfs-"
index = stdout.find(haystack)
path = stdout[index : index + len(haystack) + 16]
print(path)
