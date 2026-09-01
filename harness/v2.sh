#!/usr/bin/env bash
set -e
WORK=$HOME/ctr6
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/
cp $HOME/.cargo/bin/hyperfine $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq npm pipx >/dev/null 2>&1
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\nnpm\npipx\n' > /cfg/priority
printf 'apt:bash\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1

echo "### exit codes on a clean, converged setup (0 expected everywhere)"
for v in "check" "check drift" "list" "eval" "vars" "plan" "why bash" "adapters" "protected" "policy" "info apt:bash" "export" "sbom" "status"; do
  out=$($S $v 2>&1 >/dev/null | head -2)
  printf '  %-16s rc=%s %s\n' "$v" "$?" "$(echo $out | cut -c1-90)"
done

echo
echo "### timings for the read-only verbs"
/opt/b/hyperfine --warmup 1 --runs 5 -N -i \
  --command-name 'check'    "$S check" \
  --command-name 'eval'     "$S eval" \
  --command-name 'vars'     "$S vars" \
  --command-name 'sbom'     "$S sbom" \
  --command-name 'adapters' "$S adapters" 2>&1 | grep -E "Benchmark|Time |Range"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
