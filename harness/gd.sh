#!/usr/bin/env bash
set -e
WORK=$HOME/ctr12
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority

PKGS="cowsay sl figlet"
present () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo Y || echo n; }
states () { for p in $PKGS; do printf '%s=%s ' "$p" "$(present $p)"; done; }

decl () { : > /cfg/modules/starter.txt; for p in "$@"; do echo "apt:$p" >> /cfg/modules/starter.txt; done; }
guard () { printf '[guard]\n%s\n' "$1" > /cfg/preferences.toml; }

echo "=== SETUP: declare and install 3 packages"
decl $PKGS
guard "max_removals = 10"
$S -y sync >/dev/null 2>&1
echo "    installed: $(states)"
echo

# ---------------------------------------------------------------- ceiling
echo "=== 1. max_removals ceiling (remove 3, ceiling 2) -- must REFUSE"
guard "max_removals = 2"
decl                       # declare nothing -> all 3 become removals
out=$($S -y sync 2>&1); rc=$?
echo "    exit=$rc  still installed: $(states)"
echo "    said: $(echo "$out" | grep -iE 'max_removals|refus|ceiling|mass' | head -1 | cut -c1-120)"
echo

echo "=== 2. same, with --allow-mass-removal -- ceiling is answerable, so must PROCEED"
out=$($S -y --allow-mass-removal sync 2>&1); rc=$?
echo "    exit=$rc  still installed: $(states)"
echo

echo "    (reinstalling for the next cases)"
decl $PKGS; guard "max_removals = 10"; $S -y sync >/dev/null 2>&1
echo "    installed: $(states)"
echo

# ---------------------------------------------------------------- protected
echo "=== 3. protected_packages = [cowsay] -- must REFUSE, and -y must NOT skip it"
guard 'max_removals = 10
protected_packages = ["cowsay"]'
decl sl figlet             # cowsay becomes a removal
out=$($S -y sync 2>&1); rc=$?
echo "    exit=$rc  cowsay=$(present cowsay)  (must stay Y)"
echo "    said: $(echo "$out" | grep -iE 'protect' | head -1 | cut -c1-120)"
echo "    inspector says: $($S protected cowsay 2>&1 | head -2 | tr '\n' ' ' | cut -c1-120)"
echo

echo "=== 4. --allow-mass-removal must NOT override a protected rule"
out=$($S -y --allow-mass-removal sync 2>&1); rc=$?
echo "    exit=$rc  cowsay=$(present cowsay)  (must still be Y)"
echo

echo "=== 5. glob: protected_packages = [cow*] -- must REFUSE cowsay"
guard 'max_removals = 10
protected_packages = ["cow*"]'
out=$($S -y sync 2>&1); rc=$?
echo "    exit=$rc  cowsay=$(present cowsay)  (must stay Y)"
echo "    inspector says: $($S protected cowsay 2>&1 | head -2 | tr '\n' ' ' | cut -c1-120)"
echo

echo "=== 6. NON-glob prefix: protected_packages = [cow] -- documented NOT to cover cowsay"
guard 'max_removals = 10
protected_packages = ["cow"]'
echo "    inspector says: $($S protected cowsay 2>&1 | head -2 | tr '\n' ' ' | cut -c1-120)"
out=$($S -y sync 2>&1); rc=$?
echo "    exit=$rc  cowsay=$(present cowsay)  (n = removed, which is the documented behaviour)"
echo

echo "    (reinstalling)"
decl $PKGS; guard "max_removals = 10"; $S -y sync >/dev/null 2>&1; echo "    installed: $(states)"
echo

# ---------------------------------------------------------------- escape hatch
echo "=== 7. unprotected_packages must beat protected_packages"
guard 'max_removals = 10
protected_packages = ["cowsay"]
unprotected_packages = ["cowsay"]'
decl sl figlet
echo "    inspector says: $($S protected cowsay 2>&1 | head -2 | tr '\n' ' ' | cut -c1-120)"
out=$($S -y sync 2>&1); rc=$?
echo "    exit=$rc  cowsay=$(present cowsay)  (n = correctly removed via the escape hatch)"
echo

# ---------------------------------------------------------------- defaults / OS essential
echo "=== 8. a built-in default (bash) -- adopt it, undeclare it, must REFUSE"
decl $PKGS bash
guard "max_removals = 10"
$S -y sync >/dev/null 2>&1 || true
decl $PKGS                 # bash becomes a removal
out=$($S -y sync 2>&1); rc=$?
echo "    exit=$rc  bash=$(present bash)  (must stay Y)"
echo "    said: $(echo "$out" | grep -iE 'protect|essential' | head -1 | cut -c1-120)"
echo "    inspector says: $($S protected bash 2>&1 | head -2 | tr '\n' ' ' | cut -c1-120)"
echo

echo "=== 9. an OS-essential package apt flags (coreutils)"
echo "    inspector says: $($S protected apt:coreutils 2>&1 | head -2 | tr '\n' ' ' | cut -c1-120)"
echo "    inspector says (dpkg): $($S protected apt:dpkg 2>&1 | head -2 | tr '\n' ' ' | cut -c1-120)"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
