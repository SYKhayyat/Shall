#!/usr/bin/env bash
cat > /tmp/sha.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
apt-get update -qq >/dev/null 2>&1; apt-get install -y -qq curl ca-certificates >/dev/null 2>&1
S="/opt/b/shall --config-dir /cfg --data-dir /data"
mkdir -p /cfg /data; $S init >/dev/null 2>&1
printf 'web\n' > /cfg/priority          # web IS in priority this time
printf '[guard]\nmax_removals = 200\n' > /cfg/preferences.toml
reset () { : > /cfg/modules/starter.txt; timeout 60 $S -y --allow-mass-removal sync >/dev/null 2>&1; }

URL="https://raw.githubusercontent.com/rust-lang/rust/1.0.0/README.md"
GOOD=$(curl -fsSL "$URL" | sha256sum | cut -d' ' -f1)
WRONG=$(echo "$GOOD" | rev)
echo "real sha256: $GOOD"

echo
echo "=== A. correct sha256 must SUCCEED ==="
reset
printf 'web:%s@sha256=%s\n' "$URL" "$GOOD" > /cfg/modules/starter.txt
out=$(timeout 90 $S -y sync 2>&1); echo "  rc=$?  $(echo "$out"|grep -iE 'error|fail|sha'|head -1|cut -c1-90)"

echo "=== B. WRONG sha256 must FAIL, and say why (not a generic error) ==="
reset
printf 'web:%s@sha256=%s\n' "$URL" "$WRONG" > /cfg/modules/starter.txt
out=$(timeout 90 $S -y sync 2>&1); echo "  rc=$?"
echo "$out" | grep -iE 'sha|hash|checksum|mismatch|expected|integrity|match' | head -2 | sed 's/^/    /'

echo "=== C. NO sha256 on https -- default policy? ==="
reset
printf 'web:%s\n' "$URL" > /cfg/modules/starter.txt
out=$(timeout 90 $S -y sync 2>&1); echo "  rc=$?  $(echo "$out"|grep -iE 'sha|checksum|unverified|require'|head -1|cut -c1-95)"

echo "=== D. @unverified opt-out on https -- must SUCCEED without a hash ==="
reset
printf 'web:%s@unverified\n' "$URL" > /cfg/modules/starter.txt
out=$(timeout 90 $S -y sync 2>&1); echo "  rc=$?  (0 = opt-out honoured)"
reset
INNER
mkdir -p ~/bmnt && cp ~/shallbench/target/release/shall ~/bmnt/
docker run --rm -v /tmp/sha.sh:/sha.sh:ro -v $HOME/bmnt:/opt/b ubuntu:24.04 bash /sha.sh
