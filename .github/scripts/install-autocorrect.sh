#!/bin/sh
# 从 GitHub Release 安装 autocorrect 到 /usr/local/bin，并校验压缩包的 sha256。
#
# 用法：install-autocorrect.sh <版本> [目标]，目标默认 linux-amd64。

set -eu

version=${1:?需要 autocorrect 版本}
target=${2:-linux-amd64}

base="https://github.com/huacnlee/autocorrect/releases/download/v${version}"
tarball="/tmp/autocorrect-${target}.tar.gz"

curl -fsSL -o "$tarball" "$base/autocorrect-${target}.tar.gz"
expected=$(curl -fsSL "$base/autocorrect-${target}.tar.gz.sha256")
actual=$(sha256sum "$tarball" | cut -d' ' -f1)
if [ "$expected" != "$actual" ]; then
    echo "autocorrect 压缩包校验和不匹配：期望 $expected，实际 $actual" >&2
    exit 1
fi

sudo tar -xzf "$tarball" -C /usr/local/bin autocorrect
