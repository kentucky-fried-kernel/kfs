with open("./out", "r") as f:
    s = f.read()

allocations = {}

for l in s.splitlines():
    if l.startswith("FREE"):
        ty, addr = l.split(" ")
        addr = int(addr, 16)
        if ty in allocations:
            allocations[ty].append(addr)
        else:
            allocations[ty] = [addr]
    else:
        ty, start, end = l.split(" ")
        end = int(end, 16)
        start = int(start, 16)
        if ty in allocations:
            allocations[ty].append((start, end))
        else:
            allocations[ty] = [(start, end)]


min_slab, max_slab = 1 << 33, 0

ranges = [(a[0], a[1], i) for a, i in zip(allocations["SLAB_INIT"], range(9))]


print(f"Slab allocation range: {hex(min_slab)} - {hex(max_slab)}")

for alloc in allocations["BUDDY"]:
    start, end = alloc[0], alloc[1]
    for r in ranges:
        if r[0] < end and start < r[1]:
            print(f"Buddy overlaps with slab range: {hex(start)} - {hex(end)}")

for alloc in allocations["FREEB"]:
    for r in ranges:
        if r[0] <= alloc <= r[1]:
            print(f"Freed {hex(alloc)} from buddy that overlaps with slab[{r[2]}]: {hex(r[0])} - {hex(r[1])}")
