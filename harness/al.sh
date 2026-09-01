#!/usr/bin/env bash
# Drive a real lifecycle for many backends inside the tools image, and ask the B0 question
# for each: after Shall removes it, does `shall list` still claim it is installed?
cat > /tmp/inner_life.sh <<'INNER'
#!/usr/bin/env bash
export PATH=$PATH:/opt/b
S="/opt/b/shall --config-dir /cfg --data-dir /data"
mkdir -p /cfg /data
$S init >/dev/null 2>&1
# use every ready backend
printf 'apt\ncargo\nnpm\npipx\ngem\ngo\ncomposer\nluarocks\npip\ndotnet\nasdf\nmise\n' > /cfg/priority
printf '[guard]\nmax_removals = 200\n' > /cfg/preferences.toml

# what SHALL believes (from its lister)
believes () { $S list -b "$1" 2>/dev/null | grep -qiE "\b$2\b" && echo Y || echo n; }

life () { # $1 backend  $2 package
  local b="$1" p="$2"
  printf '%s:%s\n' "$b" "$p" > /cfg/modules/starter.txt
  timeout 400 $S -y sync >/dev/null 2>&1
  local ins=$(believes "$b" "$p")
  : > /cfg/modules/starter.txt
  timeout 400 $S -y --allow-mass-removal sync >/dev/null 2>&1
  local still=$(believes "$b" "$p")
  local verdict="ok"
  [ "$ins" != Y ] && verdict="install not confirmed by lister"
  [ "$ins" = Y ] && [ "$still" = Y ] && verdict="** B0 SHAPE: lister still lists after removal **"
  printf '  %-10s %-24s installed=%s  after-remove-lists=%s   %s\n' "$b" "$p" "$ins" "$still" "$verdict"
  : > /cfg/modules/starter.txt; timeout 200 $S -y --allow-mass-removal sync >/dev/null 2>&1
}

echo "backend    package                  installed  after-remove  verdict"
echo "----------------------------------------------------------------------------------"
life cargo    hexyl
life npm      is-odd
life pipx     pycowsay
life gem      colorize
life go       github.com/rakyll/hey
life composer psr/log
life luarocks luafilesystem
life pip      six
life dotnet   dotnetsay
life apt      figlet
INNER
mkdir -p ~/bmnt && cp ~/shallbench/target/release/shall ~/bmnt/ && docker run --rm -v /tmp/inner_life.sh:/inner.sh:ro -v /home/administrator/bmnt:/opt/b --entrypoint bash shall-tools bash /inner.sh
