import sys, re, subprocess
f = sys.argv[1]; N = int(sys.argv[2]); NF = int(sys.argv[3]); kind = int(sys.argv[4]) if len(sys.argv) > 4 else 0
maps = []; ents = []
for l in open(f):
    if l.startswith("M "):
        p = l.split(); a, b = p[1].split("-"); maps.append((int(a, 16), int(b, 16), int(p[3], 16), p[-1]))
    elif l.startswith("E "):
        h, rest = l.split(" :"); kv = dict(x.split("=") for x in h.split()[1:])
        if int(kv["k"]) == kind: ents.append((int(kv["bytes"]), int(kv["cnt"]), int(kv["max"]), [int(x, 16) for x in rest.split()]))
ents.sort(reverse=True)
def sym(a):
    for lo, hi, off, path in maps:
        if lo <= a < hi:
            if "rustc_driver" not in path and "libLLVM" not in path: return [path.split("/")[-1]]
            o = subprocess.run(["addr2line", "-i", "-f", "-C", "-e", path, hex(a - lo + off - 1)], capture_output=True, text=True).stdout.split("\n")
            return [re.sub(r"<rustc_next_trait_solver::solve::eval_ctxt::EvalCtxt<[^>]*>[^>]*>", "<EvalCtxt..>", x)[:150] for x in o[0::2] if x]
    return [hex(a)]
for b, c, mx, bt in ents[:N]:
    print("bytes=%.1fMB cnt=%d avg=%dKB max=%d" % (b / 1e6, c, b // c // 1024, mx))
    for a in bt[:NF]:
        print("   ", " | ".join(sym(a)[:3]))
