#!/bin/bash
# M0 gate —— 完成定义是 docs/milestones/m0.md 的 Gate 段，本脚本就是它的可执行形式。
# 每一项都打印证据；全部通过才 exit 0。
set -euo pipefail
cd "$(dirname "$0")/.."

TMP="$(mktemp -t oxidura-verify-m0)"
trap 'rm -f "${TMP}"' EXIT

TOTAL=0
FAILED=0

pass() {
  echo "  ok    $1"
}

fail() {
  echo "  FAIL  $1"
  if [ -s "${TMP}" ]; then
    sed 's/^/          /' "${TMP}" | tail -25
  fi
  FAILED=$((FAILED + 1))
}

run() { # run <描述> <命令...>
  TOTAL=$((TOTAL + 1))
  local desc="$1"
  shift
  if "$@" >"${TMP}" 2>&1; then pass "${desc}"; else fail "${desc}"; fi
}

check() { # check <描述> <谓词函数...>
  TOTAL=$((TOTAL + 1))
  local desc="$1"
  shift
  if "$@" >"${TMP}" 2>&1; then pass "${desc}"; else fail "${desc}"; fi
}

# core 必须零依赖：不认识数据库，也不认识 async runtime（ADR-0004）。
core_has_no_engine_deps() {
  ! grep -E 'sqlx|tokio' oxidura-core/Cargo.toml
}

# 脚本必须能在本机唯一的 /bin/bash 3.2 下解析（m0.md 的脚本形态约束）。
scripts_parse_ok() {
  local f
  for f in scripts/*.sh; do
    /bin/bash -n "${f}" || return 1
  done
  return 0
}

# 契约文档齐全（ADR-0015：文档是 gate 的一部分）。
contract_docs_present() {
  [ -f AGENTS.md ] || return 1
  [ -f docs/DESIGN.md ] || return 1
  [ "$(ls docs/adr/*.md 2>/dev/null | wc -l | tr -d ' ')" = "16" ] || return 1
  [ "$(ls docs/concepts/*.md 2>/dev/null | wc -l | tr -d ' ')" = "11" ] || return 1
  [ "$(ls docs/milestones/*.md 2>/dev/null | wc -l | tr -d ' ')" = "7" ] || return 1
  return 0
}

echo "M0 gate（docs/milestones/m0.md）"

run "cargo build --workspace" cargo build --workspace
run "cargo clippy --workspace --all-targets -- -D warnings" cargo clippy --workspace --all-targets -- -D warnings
run "cargo fmt --check" cargo fmt --check
run "cargo nextest run --workspace --no-tests=pass" cargo nextest run --workspace --no-tests=pass

# DATABASE_URL 的唯一出处是 dev-pg.sh；本地的 .env 若存在则优先。
if [ -f .env ]; then
  set -a
  . ./.env
  set +a
fi
DATABASE_URL="${DATABASE_URL:-$(scripts/dev-pg.sh url)}"
export DATABASE_URL

run "scripts/dev-pg.sh start" scripts/dev-pg.sh start
run "psql \"\$DATABASE_URL\" -c 'select 1'" psql "${DATABASE_URL}" -c 'select 1'

check "oxidura-core/Cargo.toml 里没有数据库 / async runtime 依赖" core_has_no_engine_deps
check "scripts/*.sh 在 /bin/bash（3.2）下语法正确" scripts_parse_ok
check "契约文档齐全（DESIGN + 16 ADR + 11 concepts + 7 milestones）" contract_docs_present

echo
if [ "${FAILED}" -eq 0 ]; then
  echo "M0 gate: ${TOTAL} 项全部通过"
  exit 0
fi
echo "M0 gate: ${FAILED}/${TOTAL} 项失败"
exit 1
