#!/usr/bin/env bash
set -e
WORK=$HOME/ctr32
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\nservice\nsetting\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\nmax_extra_removals = 50\n' > /cfg/preferences.toml

echo "### the machine: no init system, no settings store"
echo "  systemctl : $(command -v systemctl || echo absent)"
echo "  pid 1     : $(cat /proc/1/comm 2>/dev/null)"
echo "  gsettings : $(command -v gsettings || echo absent)"
echo "  dconf     : $(command -v dconf || echo absent)"
echo

probe () { # $1 label, $2 line
  : > /cfg/modules/starter.txt
  printf '%s\n' "$2" > /cfg/modules/starter.txt
  out=$($S -y sync 2>&1); rc=$?
  echo "### $1"
  echo "    line : $2"
  echo "    rc   : $rc"
  echo "    status line: $(echo "$out" | grep -iE 'Status:' | head -1 | tr -s ' ')"
  echo "    said : $(echo "$out" | grep -viE '^\s*$|Transaction Summary|Time:|Installs:|Removals:|====' | tail -3 | tr '\n' ' ' | cut -c1-200)"
  echo "    then check says: $($S check 2>&1 | grep -iE 'drift|health' | head -2 | tr '\n' ' ' | cut -c1-150)"
  echo
}

probe "service: on a box with no init"     'service:nginx@enabled=true'
probe "setting: on a box with no store"    'setting:org.gnome.desktop.interface/clock-format@value=24h'
probe "firewall: on a box with no fw tool" 'firewall:22/tcp'
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
