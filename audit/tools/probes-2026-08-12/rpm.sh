#!/usr/bin/env bash
mkdir -p ~/ctr25
cat > ~/ctr25/inner.sh <<'INNER'
#!/usr/bin/env bash
echo "rpm present: $(command -v rpm || echo NO)"
dnf install -y nano >/dev/null 2>&1
echo "after install   -> rpm -q nano : $(rpm -q nano 2>&1 | head -1)"
echo "                   in rpm -qa  : $(rpm -qa | grep -c '^nano-')"
dnf remove -y nano >/dev/null 2>&1
echo "after dnf remove:"
echo "  rpm -q nano         : $(rpm -q nano 2>&1 | head -1)"
echo "  in rpm -qa (lister) : $(rpm -qa | grep -c '^nano-')   <- 0 means the lister is honest"
echo "  binary on disk      : $(command -v nano || echo gone)"
echo "  config left behind  : $(ls /etc/nanorc 2>/dev/null || echo none)"
INNER
docker run --rm -v $HOME/ctr25/inner.sh:/i.sh:ro fedora:41 bash /i.sh
