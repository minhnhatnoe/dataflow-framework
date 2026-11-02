import subprocess,re
p='dataflow-flat/src/plan.rs'
old=subprocess.check_output(['git','show','HEAD:'+p],text=True)
a=old.index('    const fn excl_group(')
b=old.index('    /// Pairwise check of')
fn=old[a:b].replace('excl_group','excl_group_old')
s=open(p).read()
s=s.replace('    /// Pairwise check of',fn+'    /// Pairwise check of',1)
# counters: use dbg-free; compare in cond_comp
cur='''            if !self.excl_group(0, n) {
                self.fail(name, " joins branches that can fire in the same step", "");
                return;
            }'''
new='''            let r_new = self.excl_group(0, n);
            let mut q = 0;
            while q < n {
                self.sc_ord[q] = self.sc_offs[q];
                q += 1;
            }
            let r_old = self.excl_group_old(0, n);
            if r_new != r_old {
                self.fail(name, " VERDICT MISMATCH old/new excl_group", "");
                return;
            }
            if !r_new {
                self.fail(name, " joins branches that can fire in the same step", "");
                return;
            }'''
assert cur in s
s=s.replace(cur,new)
open(p,'w').write(s)
