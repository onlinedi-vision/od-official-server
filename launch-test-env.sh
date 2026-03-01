#!/usr/bin/env bash
#
# This is the shell script used for testing
# the od-official-server locally.
#
# Run it as follows:
#   ./launch-test-env.sh
#
# Or try -h for usage prompt:
#   ./launch-test-env.sh -h
#
# This should launch everything needed including
# e2e and unit tests.

set -eu

PIDS_FILE=".pids"
COMPOSE_FILE="test-env-compose/compose.yaml"
SCYLLA_INET="$(hostname | xargs echo -n)"
export SCYLLA_INET="${SCYLLA_INET}"
export API_PORT=1313
export NO_OF_WORKERS=32
export EXECUTABLE_NAME="api"

if ! [ -e "$PIDS_FILE" ]; then
  touch "$PIDS_FILE"
fi

function cleanup() {
  echo "======================= CLEANING UP ========================"

  echo " * killing scylla container * "
  docker compose -f "${COMPOSE_FILE}" down -v

  echo " * cleaning up mounted directories * "
  if [ -d test-env-compose/grafana ] ; then
    sudo rm -rf test-env-compose/{grafana/,prometheus_data/,prometheus/}
  fi
}

function print_usage() {
  echo "Usage: ./launch-test-env.sh [OPTION]"
  echo "OPTION:"
  echo "   -c          clean the environment (stop processes / kill docker) after everything"
  echo "               is done working"
  echo "   -C          cleanup all processes and quit"
  echo "   -h          display this message" 
  echo "   -S          skip scylla DB creation (only use if it already exists)"
  echo "   -s          skip *all* tests, only run programs"
  echo "   -u          skip *unit* tests, only run programs"
  echo "   -p <PORT>   port on which the api should run"
  echo "   -a <NAME>   change name of compiled binary / process (experimental)"
  echo "   -t <SECS>   how many seconds to wait for scylla container to settle (experimental)"
  echo "   -T <SECS>   how many seconds to wait for API program to settle (experimental)"
  echo "   -G          launch Grafana (and prometheus)"
  echo "   -e          skip end2end tests"
  echo "   -k          skip cargo target (k)aching"
}

trap 'cleanup' SIGINT SIGTERM

c_flag=''
s_flag=''
S_flag=''
u_flag=''
G_flag=''
e_flag=''
k_flag=''
scylla_wait_time=''
api_wait_time=''
while getopts 't:T:a:chsCSuGp:ek' flag; do
  case "${flag}" in
    k) k_flag='true' ;;
    e) e_flag='true' ;;
    c) c_flag='true' ;;
    C) cleanup && exit 0 || exit 1;;
    s) s_flag='true' ;;
    t) scylla_wait_time="${OPTARG:-5}" ;;
    T) api_wait_time="${OPTARG:-1}" ;;
    a) export EXECUTABLE_NAME="${OPTARG:-api}";;
    S) S_flag='true' ;;
    u) u_flag='true' ;;
    p) export API_PORT="${OPTARG:-1313}" ;;
    G) G_flag='true' ;;
    h) print_usage 
       exit 0 ;;
    *) print_usage
       exit 1 ;;
  esac
done

function launch_grafana() {
  if [[ "${G_flag}" == "true" ]]; then
    if stat grafana/ > /dev/null 2> /dev/null ; then
      sudo rm -rf test-env-compose/{grafana/,prometheus_data/,prometheus/}
    fi
  
    echo " * creating grafana mounts"
    mkdir -p test-env-compose/grafana/provisioning/datasources
    mkdir -p test-env-compose/grafana/provisioning/dashboards
    mkdir -p test-env-compose/grafana/dashboards

    echo " * creating prometheus mounts"
    mkdir -p test-env-compose/prometheus_data
    mkdir -p test-env-compose/prometheus

    cp test-env-compose/{rootfs/prometheus/prometheus.yaml,prometheus/prometheus.yml}
    cp test-env-compose/{rootfs/grafana/dashboards.yaml,grafana/provisioning/dashboards/dashboards.yml}
    cp test-env-compose/{rootfs/grafana/prometheus.yaml,grafana/provisioning/datasources/prometheus.yml}
    cp test-env-compose/{rootfs/grafana/api-dashboard.json,grafana/dashboards/api-dashboard.json}
  
    docker compose -f "${COMPOSE_FILE}" up -d prometheus
    docker compose -f "${COMPOSE_FILE}" up -d grafana
  
  fi
}

if ! [[ "${S_flag}" == "true" ]]; then
  echo "======================= LAUNCHING SCYLLA ======================="
  ./launch-scylla-locally.sh "${scylla_wait_time}"
fi

echo "======================= LAUNCHING API ========================="
echo " * api is lauching on 127.0.0.1:${API_PORT} * "

if ! [[ "${u_flag}" == "true" ]]; then
  if ! [[ "${s_flag}" == "true" ]]; then
    export RUN_UT="true"
  fi
fi

docker compose -f "${COMPOSE_FILE}" build od-official-server 
docker compose -f "${COMPOSE_FILE}" up -d od-official-server
sleep "${api_wait_time:-1}"


if ! [[ "${k_flag}" == "true" ]]; then
  echo "====================== CACHING COMPILATION ========================="
  [ -d ./taget ] && rm -rf ./target

  mkdir -p ./target/release
  mkdir -p ./target/debug

  docker cp od-official-server:/build/target/release ./target
  docker cp od-official-server:/build/target/debug ./target || true
fi

echo "======================= LAUNCHING GRAFANA ======================="
launch_grafana

if [[ "${s_flag}" == "true" ]] || [[ "${e_flag}" == "true" ]]; then
  echo "====================== SKIPPING E2E TESTS ========================="
  if [[ "${c_flag}" == "true" ]]; then
    cleanup
  fi
  exit 0
fi

echo "================== LAUNCHING API E2E TESTS ===================="
./shadow/e2e.sh "http://localhost:${API_PORT}"

if [[ "${c_flag}" == "true" ]]; then
  cleanup
fi

