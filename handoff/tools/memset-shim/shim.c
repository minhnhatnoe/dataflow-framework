#define _GNU_SOURCE
#include <dlfcn.h>
#include <execinfo.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <sys/mman.h>
#include <pthread.h>
#define TH (64*1024)
#define NB 4096
#define DEPTH 28
typedef struct { void *bt[DEPTH]; int kind; unsigned long cnt, bytes, max; } Ent;
static Ent tab[NB];
static pthread_mutex_t mu = PTHREAD_MUTEX_INITIALIZER;
static __thread int guard;
static void *(*real_memset)(void*,int,size_t);
static int (*real_madvise)(void*,size_t,int);
static unsigned long hist[2][40]; static unsigned long tot[4];
typedef struct{size_t n;int c;void*a,*b;}Ev; static Ev evs[4000]; static int nev;
static void rec(int kind, size_t n, int c){
  if(guard) return; guard=1;
  void *bt[DEPTH+2]; int d=backtrace(bt,DEPTH+2);
  unsigned long h=kind; for(int i=2;i<d;i++) h=h*1000003u^(uintptr_t)bt[i];
  pthread_mutex_lock(&mu);
  if(kind==0 && nev<4000){ static unsigned long k0; if(++k0%25==0){evs[nev].n=n;evs[nev].c=c;evs[nev].a=bt[2];evs[nev].b=bt[3];nev++;}}
  int lg=63-__builtin_clzl(n); hist[kind][lg]++; tot[kind]+=n;
  for(int p=0;p<NB;p++){ Ent*e=&tab[(h+p)%NB];
    if(e->cnt==0){ memset(e->bt,0,sizeof e->bt); for(int i=2;i<d;i++) e->bt[i-2]=bt[i]; e->kind=kind; e->cnt=1; e->bytes=n; e->max=n; break; }
    void *k[DEPTH]; memset(k,0,sizeof k); for(int i=2;i<d;i++) k[i-2]=bt[i];
    if(e->kind==kind && !memcmp(e->bt,k,sizeof k)){ e->cnt++; e->bytes+=n; if(n>e->max)e->max=n; break;} }
  pthread_mutex_unlock(&mu); guard=0;
}
void *memset(void*s,int c,size_t n){
  if(!real_memset) real_memset=dlsym(RTLD_NEXT,"memset");
  if(n>=TH) rec(0,n,c);
  return real_memset(s,c,n);
}
int madvise(void*a,size_t n,int adv){
  if(!real_madvise) real_madvise=dlsym(RTLD_NEXT,"madvise");
  if(n>=TH) rec(1,n,adv);
  return real_madvise(a,n,adv);
}
__attribute__((destructor)) static void dump(void){
  guard=1; char fn[256]; snprintf(fn,sizeof fn,"%s/out.%d",getenv("MSDIR")?getenv("MSDIR"):".",getpid());
  FILE*f=fopen(fn,"w"); if(!f) return;
  fprintf(f,"TOTAL memset_bytes=%lu madvise_bytes=%lu\n",tot[0],tot[1]);
  for(int k=0;k<2;k++){ fprintf(f,"HIST %d",k); for(int i=0;i<40;i++) if(hist[k][i]) fprintf(f," 2^%d:%lu",i,hist[k][i]); fprintf(f,"\n"); }
  for(int i=0;i<NB;i++) if(tab[i].cnt){ fprintf(f,"E k=%d cnt=%lu bytes=%lu max=%lu :",tab[i].kind,tab[i].cnt,tab[i].bytes,tab[i].max); for(int j=0;j<DEPTH&&tab[i].bt[j];j++) fprintf(f," %p",tab[i].bt[j]); fprintf(f,"\n"); }
  for(int i=0;i<nev;i++) fprintf(f,"V %zu %d %p %p\n",evs[i].n,evs[i].c,evs[i].a,evs[i].b);
  FILE*m=fopen("/proc/self/maps","r"); char l[512]; while(fgets(l,sizeof l,m)) if(strstr(l,"r-xp")) fprintf(f,"M %s",l);
  fclose(m); fclose(f);
}
