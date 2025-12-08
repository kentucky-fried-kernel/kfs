with open("./out", "r") as f:
    s = f.read()

allocations = {}

for l in s.splitlines():
    ty, addr, size = l.split(" ")
    size = int(size, 10)
    addr = int(addr, 16)
    if ty in allocations:
        allocations[ty].append((addr, size))
    else:
        allocations[ty] = [(addr, size)]
    # print(ty, hex(addr), size)

min_slab, max_slab = 1 << 33, 0

ranges = [(a[0], a[1]) for a in allocations["SLAB_INIT"]]


print(f"Slab allocation range: {hex(min_slab)} - {hex(max_slab)}")

for alloc in allocations["BUDDY"]:
    start, end = alloc[0], alloc[0] + alloc[1]
    for r in ranges:
        if r[0] <= end and start <= r[1]:
            print(f"Buddy overlaps with slab range: {hex(start)} - {hex(end)}")
