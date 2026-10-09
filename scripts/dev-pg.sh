#!/bin/bash
# 本机开发用的常驻 PostgreSQL 集群：用 Postgres.app 的二进制，数据目录在仓库外。
#
# 端口刻意不用 5432 —— 那是 Postgres.app 自带集群的端口，两边不要互相踩。
# 端口 / 用户 / 库名的唯一出处就是这个脚本：需要 DATABASE_URL 就用 `dev-pg.sh url` 拿。
#
# 用法：scripts/dev-pg.sh {start|stop|status|url|psql|logs} [psql 参数...]
#
# 两个本机 bash 3.2 的坑（写新脚本时一样适用）：
#   1. 变量名后面紧跟中文字符（或全角括号）时，必须写 ${VAR}。C locale 下 bash 会把
#      UTF-8 字节并进变量名，$PGPORT）会变成找变量 `PGPORT）`，set -u 直接报 unbound。
#   2. 不用 4.x 才有的构造（declare -A / mapfile / ${var,,} / &>>）。
set -euo pipefail

PG_HOME="${OXIDURA_PG_HOME:-/Applications/Postgres.app/Contents/Versions/latest/bin}"
PGDATA="${OXIDURA_PGDATA:-$HOME/.local/share/oxidura/pgdata}"
PGPORT="${OXIDURA_PGPORT:-55432}"
DBUSER="${OXIDURA_PGUSER:-oxidura}"
DBNAME="${OXIDURA_PGDATABASE:-oxidura}"
URL="postgres://${DBUSER}@127.0.0.1:${PGPORT}/${DBNAME}"
LOG="${PGDATA}/postgres.log"

PG_CTL="${PG_HOME}/pg_ctl"
PSQL="${PG_HOME}/psql"
INITDB="${PG_HOME}/initdb"
CREATEDB="${PG_HOME}/createdb"

die() {
  echo "dev-pg: $*" >&2
  exit 1
}

require_bins() {
  local t
  for t in "${PG_CTL}" "${PSQL}" "${INITDB}" "${CREATEDB}"; do
    [ -x "${t}" ] || die "找不到 ${t}（用 OXIDURA_PG_HOME 指向 Postgres.app 的 bin 目录）"
  done
}

is_running() {
  [ -s "${PGDATA}/postmaster.pid" ] && "${PG_CTL}" -D "${PGDATA}" status >/dev/null 2>&1
}

query_port() {
  "${PSQL}" -h 127.0.0.1 -p "${PGPORT}" -U "${DBUSER}" -d postgres -tAc "$1"
}

cmd_start() {
  require_bins
  if is_running; then
    echo "dev-pg: 已在运行 → ${URL}"
    return 0
  fi
  if [ ! -s "${PGDATA}/PG_VERSION" ]; then
    echo "dev-pg: 初始化集群 ${PGDATA}"
    mkdir -p "${PGDATA}"
    "${INITDB}" -D "${PGDATA}" -U "${DBUSER}" --auth=trust --encoding=UTF8 --locale=en_US.UTF-8 >/dev/null
  fi
  echo "dev-pg: 启动（端口 ${PGPORT}）"
  local opts="-p ${PGPORT} -k /tmp -c listen_addresses=127.0.0.1"
  if ! "${PG_CTL}" -D "${PGDATA}" -l "${LOG}" -w -t 60 -o "${opts}" start >/dev/null; then
    echo "--- ${LOG} ---" >&2
    tail -20 "${LOG}" >&2 || true
    die "启动失败"
  fi
  if [ "$(query_port "select 1 from pg_database where datname = '${DBNAME}'")" != "1" ]; then
    "${CREATEDB}" -h 127.0.0.1 -p "${PGPORT}" -U "${DBUSER}" "${DBNAME}"
    echo "dev-pg: 建库 ${DBNAME}"
  fi
  echo "dev-pg: 就绪 → ${URL}"
}

cmd_stop() {
  require_bins
  if ! is_running; then
    echo "dev-pg: 未在运行"
    return 0
  fi
  "${PG_CTL}" -D "${PGDATA}" -w -t 60 -m fast stop >/dev/null
  echo "dev-pg: 已停止"
}

cmd_status() {
  require_bins
  if is_running; then
    echo "dev-pg: 运行中 → ${URL}"
  else
    echo "dev-pg: 未运行（PGDATA=${PGDATA}）"
    exit 1
  fi
}

case "${1:-}" in
  start) cmd_start ;;
  stop) cmd_stop ;;
  status) cmd_status ;;
  url) printf '%s\n' "${URL}" ;;
  psql)
    require_bins
    shift
    exec "${PSQL}" "${URL}" "$@"
    ;;
  logs) exec tail -f "${LOG}" ;;
  *)
    cat >&2 <<'USAGE'
用法: scripts/dev-pg.sh {start|stop|status|url|psql|logs}

  start   首次会初始化集群，然后启动；重复调用是幂等的
  stop    优雅停止
  status  是否在运行（未运行时 exit 1）
  url     打印 DATABASE_URL
  psql    用该库打开 psql，其余参数原样透传
  logs    tail -f 服务端日志

环境变量:
  OXIDURA_PG_HOME      Postgres.app 的 bin 目录
  OXIDURA_PGDATA       数据目录（必须在仓库外）
  OXIDURA_PGPORT       端口，默认 55432
  OXIDURA_PGUSER       超级用户名，默认 oxidura
  OXIDURA_PGDATABASE   库名，默认 oxidura
USAGE
    exit 2
    ;;
esac
