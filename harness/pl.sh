#!/usr/bin/env bash
set -e
WORK=$HOME/ctr34
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
apt-get update -qq >/dev/null 2>&1
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\nnpm\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\nmax_extra_removals = 50\n' > /cfg/preferences.toml

# run a manifest through the PLANNER and report rc + one line of what it said
plan () { # $1 label, rest: manifest lines
  local label="$1"; shift
  rm -f /cfg/modules/*.txt
  printf '%s\n' "$@" > /cfg/modules/starter.txt
  local out rc
  out=$(timeout 60 $S -y sync --dry-run 2>&1); rc=$?
  local msg
  msg=$(echo "$out" | grep -viE '^\s*$|^Planned|^\s+backends:|^\s+privileges:|would' | tail -1 | cut -c1-115)
  [ -z "$msg" ] = && msg=$(echo "$out" | tail -1 | cut -c1-115)
  printf '  %-40s rc=%-3s %s\n' "$label" "$rc" "$msg"
}

echo "############ adversarial but VALID manifests -- the planner, not the parser"
plan "present + absent, same package"      'apt:jq' 'absent:apt:jq'
plan "same package declared twice"          'apt:jq' 'apt:jq'
plan "same package, conflicting versions"   'apt:jq@version=1.6' 'apt:jq@version=1.7'
plan "same name, two backends"              'apt:jq' 'npm:jq'
plan "backend chain"                        'apt,npm:jq'
plan "absent for a name never declared"     'absent:apt:definitely-not-here'
plan "hold + version (documented clash)"    'apt:jq@version=1.6,hold'
plan "empty manifest"                       ''
plan "comment only"                         '# nothing here'

echo
echo "############ module graph pathologies"
mk () { printf '%s\n' "${@:2}" > "/cfg/modules/$1.txt"; }
rm -f /cfg/modules/*.txt
mk a 'use b'; mk b 'use a'
printf 'use a\n' > /cfg/modules/starter.txt
out=$(timeout 60 $S -y sync --dry-run 2>&1); rc=$?
printf '  %-40s rc=%-3s %s\n' "use cycle a<->b" "$rc" "$(echo "$out" | tail -1 | cut -c1-115)"

rm -f /cfg/modules/*.txt
mk self 'use self'
printf 'use self\n' > /cfg/modules/starter.txt
out=$(timeout 60 $S -y sync --dry-run 2>&1); rc=$?
printf '  %-40s rc=%-3s %s\n' "self-use" "$rc" "$(echo "$out" | tail -1 | cut -c1-115)"

rm -f /cfg/modules/*.txt
prev=""
for i in $(seq 1 60); do if [ -z "$prev" ]; then mk "m$i" 'apt:jq'; else mk "m$i" "use $prev"; fi; prev="m$i"; done
printf 'use %s\n' "$prev" > /cfg/modules/starter.txt
out=$(timeout 60 $S -y sync --dry-run 2>&1); rc=$?
printf '  %-40s rc=%-3s %s\n' "60-deep use chain" "$rc" "$(echo "$out" | tail -1 | cut -c1-115)"

echo
echo "############ scale"
rm -f /cfg/modules/*.txt
python3 -c "
import io
io.open('/cfg/modules/starter.txt','w').write('\n'.join('apt:pkg%05d'%i for i in range(5000))+'\n')
" 2>/dev/null || for i in $(seq 1 5000); do echo "apt:pkg$i"; done > /cfg/modules/starter.txt
start=$(date +%s%N)
out=$(timeout 300 $S -y sync --dry-run 2>&1); rc=$?
end=$(date +%s%N)
printf '  %-40s rc=%-3s %sms  %s\n' "5000-package manifest" "$rc" "$(( (end-start)/1000000 ))" "$(echo "$out" | grep -oE 'install [0-9]+' | head -1)"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
