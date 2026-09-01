#!/usr/bin/env bash
set -e
WORK=$HOME/ctr28
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\nmax_extra_removals = 50\n' > /cfg/preferences.toml
mkdir -p /cfg/dotfiles
echo "SOURCE-CONTENT-v1" > /cfg/dotfiles/vimrc

printf 'link:./dotfiles/vimrc@target=/root/.vimrc\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1

echo "### the link Shall created"
ls -l /root/.vimrc
echo "   readlink        : $(readlink /root/.vimrc)"
echo "   resolves to     : $(readlink -f /root/.vimrc 2>/dev/null || echo '(cannot resolve)')"
echo "   target exists?  : $(test -e /root/.vimrc && echo YES || echo 'NO -- DANGLING')"
echo
echo "### can anything actually READ the dotfile?  (this is the whole point of link:)"
if content=$(cat /root/.vimrc 2>&1); then
  echo "   cat succeeded   : $content"
else
  echo "   cat FAILED      : $content"
fi
echo
echo "### where the source really is"
echo "   /cfg/dotfiles/vimrc          : $(test -e /cfg/dotfiles/vimrc && echo exists || echo missing)"
echo "   /root/dotfiles/vimrc (link's): $(test -e /root/dotfiles/vimrc && echo exists || echo missing)"
echo
echo "### and does Shall think this is fine?"
$S check 2>&1 | grep -iE 'drift|place|undo|ok ' | head -4 | sed 's/^/   /'
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
