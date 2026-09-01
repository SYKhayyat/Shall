#!/usr/bin/env bash
set -e
WORK=$HOME/ctr2
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq strace >/dev/null 2>&1
mkdir -p /cfg /data
/opt/b/shall --config-dir /cfg --data-dir /data init >/dev/null 2>&1

probe () {
  local label="$1"; shift
  printf '%s\n' "$@" > /cfg/priority
  strace -f -e trace=execve -o /tmp/t.txt \
    /opt/b/shall --config-dir /cfg --data-dir /data list >/dev/null 2>&1 || true
  # distinct program basenames Shall tried to exec, excluding itself
  local progs
  progs=$(grep -o 'execve("[^"]*"' /tmp/t.txt \
          | sed -E 's/execve\("//; s/"$//' \
          | xargs -n1 basename 2>/dev/null \
          | grep -v '^shall$' | sort -u)
  echo "== priority = [$*]"
  echo "   distinct binaries probed: $(echo "$progs" | grep -c . )"
  echo "$progs" | tr '\n' ' ' | fold -w 150 -s | sed 's/^/   /'
  echo
}

probe "one"  apt
probe "three" apt npm pipx
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
