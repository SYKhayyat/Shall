#!/usr/bin/env bash
set -e
WORK=$HOME/ctr35
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/fuzz2.py <<'PYEOF'
import random, subprocess, sys
BIN="/opt/b/shall"; MOD="/cfg/modules/starter.txt"
TOK = ["apt","cargo","npm","list","absent:","web:","github:","link:","service:",
 "setting:","exec:","firewall:","dotfiles:","shim:","schedule:","generate:",
 ":",",","@","=","*","!","?","#",'"',"'","\\","/","|","&",";","(",")","[","]",
 "{","}","<",">","$","`","~","^","%","+","-",".","when","use","if","else","not",
 "and","or","host","os","arch","version","hold","sha256","target","value","runs",
 "until","requires"," ","\t","  ","0","1","999999999999999999999","-1","1.2.3",
 ">=1.0","\U0001F600","\r","\x1b[31m","%s","{}","../..","a"*300]
# exit codes the program documents: 0 converged, 1 failed, 2 differences, 3 refused
OK = {0,1,2,3}
def gen(rng): return "".join(rng.choice(TOK) for _ in range(rng.randint(1,14)))
def main():
    rng=random.Random(int(sys.argv[1])); n=int(sys.argv[2])
    bad=[]
    for i in range(n):
        text="\n".join(gen(rng) for _ in range(rng.randint(1,4)))
        open(MOD,"w",encoding="utf-8",errors="replace").write(text+"\n")
        try:
            p=subprocess.run([BIN,"--config-dir","/cfg","--data-dir","/data","-y","sync","--dry-run"],
                             capture_output=True,timeout=40,text=True,errors="replace")
            rc,out=p.returncode,(p.stdout or "")+(p.stderr or "")
        except subprocess.TimeoutExpired:
            bad.append(("HANG",text,"")); continue
        low=out.lower()
        if "panicked at" in low or "rust_backtrace" in low:
            bad.append(("PANIC",text,next((l for l in out.splitlines() if "panicked" in l.lower()),"")[:200]))
        elif rc not in OK:
            bad.append(("RC=%s"%rc,text,(out.strip().splitlines() or [""])[-1][:150]))
    print("=== %d planner inputs, %d anomalies (documented codes: 0,1,2,3) ==="%(n,len(bad)))
    for k,t,m in bad[:15]: print("%s: %r\n    %s"%(k,t[:160],m))
    if not bad: print("no panics, no hangs, no undocumented exit codes")
main()
PYEOF

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq python3 >/dev/null 2>&1
mkdir -p /cfg /data
/opt/b/shall --config-dir /cfg --data-dir /data init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\nmax_extra_removals = 50\n' > /cfg/preferences.toml
python3 /opt/b/fuzz2.py 777 500
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
