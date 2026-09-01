#!/usr/bin/env bash
cat > /tmp/nb.sh <<'INNER'
#!/usr/bin/env bash
export PATH=$PATH:/opt/b
S="/opt/b/shall --config-dir /cfg --data-dir /data"
mkdir -p /cfg /data; $S init >/dev/null 2>&1
printf 'apt\nnpm\ngem\npip\n' > /cfg/priority
printf '[guard]\nmax_removals = 200\n' > /cfg/preferences.toml
reset () { : > /cfg/modules/starter.txt; timeout 300 $S -y --allow-mass-removal sync >/dev/null 2>&1; }

echo "############ 1. @version -- does it install EXACTLY that version?"
reset
printf 'npm:is-odd@version=2.0.0\n' > /cfg/modules/starter.txt
timeout 300 $S -y sync >/dev/null 2>&1
echo "  asked is-odd@2.0.0 -> npm reports: $(npm ls -g is-odd --depth=0 2>/dev/null | grep -oE 'is-odd@[0-9.]+' || echo none)"

echo "  now change the pin to 3.0.0 and re-sync (must CHANGE the installed version):"
printf 'npm:is-odd@version=3.0.0\n' > /cfg/modules/starter.txt
timeout 300 $S -y sync >/dev/null 2>&1
echo "  asked is-odd@3.0.0 -> npm reports: $(npm ls -g is-odd --depth=0 2>/dev/null | grep -oE 'is-odd@[0-9.]+' || echo none)"

echo "  gem, pinned older version then check it did NOT drift to latest:"
reset
printf 'gem:colorize@version=0.8.0\n' > /cfg/modules/starter.txt
timeout 300 $S -y sync >/dev/null 2>&1
echo "  asked colorize@0.8.0 -> gem reports: $(gem list -e colorize 2>/dev/null | grep -oE 'colorize \([0-9., ]+\)' || echo none)"
echo "  now an IDEMPOTENT re-sync -- must NOT try to bump it:"
timeout 300 $S -y sync 2>&1 | grep -iE 'install|remove|up to date|change' | head -1 | sed 's/^/    /'

echo
echo "############ 2. @sha256 -- does a WRONG hash get rejected on a download?"
reset
# a web: download with a deliberately wrong sha256 must FAIL, not install
printf 'web:https://raw.githubusercontent.com/git/git/master/README.md@sha256=0000000000000000000000000000000000000000000000000000000000000000\n' > /cfg/modules/starter.txt
out=$(timeout 200 $S -y sync 2>&1); rc=$?
echo "  wrong sha256 -> rc=$rc"
echo "  said: $(echo "$out" | grep -iE 'sha|hash|checksum|mismatch|integrity|refus' | head -1 | cut -c1-120)"

echo "  and a MISSING sha256 on a plain http (not https) URL -- allow_http off by default?"
printf 'web:http://example.com/x.txt\n' > /cfg/modules/starter.txt
out=$(timeout 120 $S -y sync 2>&1); rc=$?
echo "  http no-sha -> rc=$rc  said: $(echo "$out" | grep -iE 'http|sha|checksum|unverified|refus|https' | head -1 | cut -c1-120)"

echo
echo "############ 3. when -- does a false condition actually SUPPRESS a package?"
reset
cat > /cfg/modules/starter.txt <<'EOF'
when host == definitely-not-this-host {
  npm:left-pad
}
npm:is-odd
EOF
timeout 300 $S -y sync >/dev/null 2>&1
echo "  is-odd (unconditional) installed? $(npm ls -g is-odd --depth=0 2>/dev/null | grep -qc is-odd && echo Y || echo n)"
echo "  left-pad (false when) installed?  $(npm ls -g left-pad --depth=0 2>/dev/null | grep -q left-pad && echo 'Y -- BUG: false when did not suppress' || echo 'n -- correct')"

echo "  now a TRUE condition -- does it include?"
reset
HOST=$(hostname)
cat > /cfg/modules/starter.txt <<EOF
when host == $HOST {
  npm:is-odd
}
EOF
timeout 300 $S -y sync >/dev/null 2>&1
echo "  true when on host=$HOST -> is-odd installed? $(npm ls -g is-odd --depth=0 2>/dev/null | grep -q is-odd && echo 'Y -- correct' || echo 'n -- BUG: true when did not include')"
reset
INNER
mkdir -p ~/bmnt && cp ~/shallbench/target/release/shall ~/bmnt/
docker run --rm -v /tmp/nb.sh:/nb.sh:ro -v $HOME/bmnt:/opt/b --entrypoint /bin/bash shall-tools /nb.sh
