#!/bin/sh
# 判断定时运行的工作流这次是否需要跑：与上一次成功运行的 head_sha 比对。
#
# 用法：nightly-check.sh <工作流文件名>，例如 nightly-check.sh msrv.yml。
# 读取 GITHUB_EVENT_NAME、GITHUB_SHA、GITHUB_REF_NAME、GH_TOKEN 与 GH_REPO，把
# changed=true 或 changed=false 写到标准输出，供 $GITHUB_OUTPUT 使用。

set -eu

workflow=${1:?需要工作流文件名}

if [ "${GITHUB_EVENT_NAME:-}" != schedule ]; then
    echo "changed=true"
    exit 0
fi

branch=${GITHUB_REF_NAME:-master}
last=$(gh api \
    "repos/${GH_REPO}/actions/workflows/${workflow}/runs?branch=${branch}&status=success&per_page=1" \
    --jq '.workflow_runs[0].head_sha // ""')

if [ "$last" = "${GITHUB_SHA}" ]; then
    echo "changed=false"
else
    echo "changed=true"
fi
