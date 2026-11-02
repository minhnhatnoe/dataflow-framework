import sys, subprocess
f, addr, before = sys.argv[1], int(sys.argv[2], 16), int(sys.argv[3])
for l in open(f):
    if l.startswith("M ") and "rustc_driver" in l:
        p = l.split(); a, b = p[1].split("-"); lo, hi, off = int(a,16), int(b,16), int(p[3],16)
        if lo <= addr < hi:
            fo = addr - lo + off; path = p[-1]
            # virtual address == file offset - segment delta; use objdump with vma computed via readelf not needed if offsets equal vma+delta
            print(path, hex(fo))
            r = subprocess.run(["addr2line","-f","-C","-e",path,hex(fo-1)],capture_output=True,text=True).stdout.split("\n")[0]
            print("nearest sym", r)
            # find vma of this file offset: program headers
            ph = subprocess.run(["readelf","-lW",path],capture_output=True,text=True).stdout
            for q in ph.split("\n"):
                w = q.split()
                if w and w[0]=="LOAD":
                    o,v,fs = int(w[1],16), int(w[2],16), int(w[4],16)
                    if o <= fo < o+fs:
                        vma = fo - o + v
                        out = subprocess.run(["objdump","-d","-C","--no-show-raw-insn","-M","intel","--start-address",hex(vma-before),"--stop-address",hex(vma+8),path],capture_output=True,text=True).stdout
                        print(out[-6000:])
