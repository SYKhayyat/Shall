#!/usr/bin/env bash
set -e
WORK=$HOME/ctr33
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/fuzz.py <<'PYEOF'
import random, subprocess, os, sys

CFG, DATA = "/cfg", "/data"
BIN = "/opt/b/shall"
MOD = "/cfg/modules/starter.txt"

TOK = [
    "apt", "cargo", "npm", "list", "absent:", "web:", "github:", "link:", "service:",
    "setting:", "exec:", "firewall:", "dotfiles:", "shim:", "schedule:", "generate:",
    ":", ",", "@", "=", "*", "!", "?", "#", '"', "'", "\\", "/", "|", "&", ";",
    "(", ")", "[", "]", "{", "}", "<", ">", "$", "`", "~", "^", "%", "+", "-", ".",
    "when", "use", "if", "else", "not", "and", "or", "host", "os", "arch",
    "version", "hold", "sha256", "target", "value", "runs", "until", "requires",
    " ", "\t", "  ", "0", "1", "999999999999999999999", "-1", "1.2.3", ">=1.0",
    "Ã©", "ä¸­æ–‡", "\U0001F600", "\r", "\x1b[31m", "%s", "{}", "../..",
    "a" * 300,
]

def gen(rng, maxtok=14):
    return "".join(rng.choice(TOK) for _ in range(rng.randint(1, maxtok)))

def run(text):
    with open(MOD, "w", encoding="utf-8", errors="replace") as f:
        f.write(text + "\n")
    try:
        p = subprocess.run([BIN, "--config-dir", CFG, "--data-dir", DATA, "eval"],
                           capture_output=True, timeout=25, text=True, errors="replace")
        return p.returncode, (p.stdout or "") + (p.stderr or "")
    except subprocess.TimeoutExpired:
        return "TIMEOUT", ""

def main():
    seed = int(sys.argv[1]); n = int(sys.argv[2]); multiline = len(sys.argv) > 3
    rng = random.Random(seed)
    bad = []
    for i in range(n):
        if multiline:
            text = "\n".join(gen(rng) for _ in range(rng.randint(1, 6)))
        else:
            text = gen(rng)
        rc, out = run(text)
        low = out.lower()
        if rc == "TIMEOUT":
            bad.append(("HANG", text, ""))
        elif "panicked at" in low or "rust_backtrace" in low or "internal error" in low:
            first = next((l for l in out.splitlines() if "panicked" in l.lower()), out[:160])
            bad.append(("PANIC", text, first[:200]))
        elif rc not in (0, 1):
            bad.append(("RC=%s" % rc, text, (out.strip().splitlines() or [""])[-1][:160]))
    print("=== %d inputs (%s), %d anomalies ===" % (n, "multi-line" if multiline else "single-line", len(bad)))
    for kind, text, msg in bad[:20]:
        print("%s: %r\n    %s" % (kind, text[:200], msg))
    if not bad:
        print("no panics, no hangs, no exit code outside {0,1}")

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
echo "warm-up eval timing:"
time /opt/b/shall --config-dir /cfg --data-dir /data eval >/dev/null 2>&1
echo
python3 /opt/b/fuzz.py 1234 1200
echo
python3 /opt/b/fuzz.py 99 400 multiline
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
