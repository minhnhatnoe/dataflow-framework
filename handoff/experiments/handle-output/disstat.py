#!/usr/bin/env python3
"""disstat.py DIS.txt NAME...: instruction counts and mnemonic histograms of `<..>::push` functions in an objdump -d -C listing;
with two NAMEs also the mnemonic differences between them."""
import re, sys, collections
cur = None
ops = {}
for l in open(sys.argv[1]):
    m = re.match(r'^[0-9a-f]+ <(.*)>:$', l)
    if m:
        cur = m.group(1); ops[cur] = []; continue
    if cur and re.match(r'^\s+[0-9a-f]+:', l):
        p = l.split('\t')
        ops[cur].append(p[1].split()[0] if len(p) > 1 and p[1].split() else '')
ks = {}
for k in ops:
    mm = re.search(r'(\w+)>::push$', k)
    if mm and 'handle_proto' in k:
        ks[mm.group(1)] = k
for n, k in ks.items():
    c = collections.Counter(ops[k])
    print(n, len(ops[k]), 'mov', sum(v for o, v in c.items() if o.startswith('mov')), 'call', c['call'])
names = sys.argv[2:]
if len(names) == 2:
    ha, hb = (collections.Counter(ops[ks[n]]) for n in names)
    print({o: (ha[o], hb[o]) for o in set(ha) | set(hb) if ha[o] != hb[o]})
