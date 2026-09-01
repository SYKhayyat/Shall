#!/bin/bash
export SHALL_CONFIG_DIR=/tmp/g/cfg SHALL_DATA_DIR=/tmp/g/data
rm -rf /tmp/g; mkdir -p /tmp/g; shall init >/dev/null 2>&1
shall adopt -y >/dev/null 2>&1
echo "=== FULL check output on a converged machine ==="
shall check; echo "rc=$?"
