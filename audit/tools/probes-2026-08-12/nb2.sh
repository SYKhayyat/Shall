#!/usr/bin/env bash
cat > /tmp/nb2.sh <<'INNER'
#!/usr/bin/env bash
export PATH=$PATH:/opt/b
S="/opt/b/shall --config-dir /cfg --data-dir /data"
mkdir -p /cfg /data; $S init >/dev/null 2>&1
printf 'apt\nnpm\ngem\n' > /cfg/priority
printf '[guard]\nmax_removals = 200\n' > /cfg/preferences.toml
reset () { : > /cfg/modules/starter.txt; timeout 200 $S -y --allow-mass-removal sync >/dev/null 2>&1; }

echo "############ 1. sha256: does the RIGHT hash pass and a WRONG hash fail on the SAME url?"
URL="https://raw.githubusercontent.com/rust-lang/rust/1.0.0/README.md"
GOOD=$(curl -fsSL "$URL" 2>/dev/null | sha256sum | cut -d' ' -f1)
echo "  real sha256 of the file: $GOOD"
reset
printf 'web:%s@sha256=%s\n' "$URL" "$GOOD" > /cfg/modules/starter.txt
out=$(timeout 120 $S -y sync 2>&1); echo "  CORRECT hash -> rc=$? ($(echo "$out" | grep -icE 'error|fail') error lines)"
reset
printf 'web:%s@sha256=%s\n' "$URL" "$(echo $GOOD | tr 'a-f0-9' '0-9a-f')" > /cfg/modules/starter.txt
out=$(timeout 120 $S -y sync 2>&1)
echo "  WRONG hash   -> rc=$?  message:"
echo "$out" | grep -iE 'sha|hash|checksum|mismatch|integrity|expected|got' | head -2 | sed 's/^/     /'

echo
echo "############ 2. cross-backend priority: bare name both managers have"
reset
# both npm and gem have a package called 'json'. priority lists npm first.
printf 'npm\ngem\n' > /cfg/priority
printf 'json\n' > /cfg/modules/starter.txt
timeout 200 $S -y sync >/dev/null 2>&1
echo "  bare 'json', priority npm>gem -> resolved to: $($S why json 2>&1 | grep -oiE '(npm|gem):json' | head -1 || $S list 2>/dev/null | grep -iE '\bjson\b' | head -1 | awk '{print $1}')"
echo "  reversed priority gem>npm:"
reset
printf 'gem\nnpm\n' > /cfg/priority
printf 'json\n' > /cfg/modules/starter.txt
timeout 200 $S -y sync >/dev/null 2>&1
echo "  bare 'json', priority gem>npm -> resolved to: $($S list 2>/dev/null | grep -iE '\bjson\b' | head -1 | awk '{print $1}')"

echo
echo "############ 3. @hold actually blocks a bulk upgrade?"
reset
printf 'npm\n' > /cfg/priority
printf 'npm:is-odd@version=2.0.0\nnpm:is-even@version=1.0.0\n' > /cfg/modules/starter.txt
timeout 200 $S -y sync >/dev/null 2>&1
echo "  before: is-odd=$(npm ls -g is-odd --depth=0 2>/dev/null|grep -oE 'is-odd@[0-9.]+') is-even=$(npm ls -g is-even --depth=0 2>/dev/null|grep -oE 'is-even@[0-9.]+')"
# hold is-odd, then upgrade -- is-odd must stay, is-even may move
printf 'npm:is-odd@hold\nnpm:is-even\n' > /cfg/modules/starter.txt
timeout 200 $S -y sync >/dev/null 2>&1
timeout 300 $S -y upgrade 2>&1 | tail -2 | sed 's/^/    /'
echo "  after upgrade: is-odd=$(npm ls -g is-odd --depth=0 2>/dev/null|grep -oE 'is-odd@[0-9.]+') (held, must stay 2.0.0)"
reset
INNER
mkdir -p ~/bmnt && cp ~/shallbench/target/release/shall ~/bmnt/
docker run --rm -v /tmp/nb2.sh:/nb2.sh:ro -v $HOME/bmnt:/opt/b --entrypoint /bin/bash shall-tools /nb2.sh
