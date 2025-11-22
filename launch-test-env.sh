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

function cleanup() {
  echo "======================= CLEANING UP ========================"
  echo " * killing scylla container * "
  docker kill scylla-division-online
  echo " * killing api process * "
  kill -9 "$(ps -e | pgrep \"${EXECUTABLE_NAME:-api}\")"
}

function print_usage() {
  echo "Usage: ./launch-test-env.sh [OPTION]"
  echo "OPTION:"
  echo "   -c          clean the environment (stop processes / kill docker)"
  echo "   -h          display this message" 
  echo "   -S          skip scylla DB creation (only use if it already exists)"
  echo "   -s          skip *all* tests, only run programs"
  echo "   -u          skip *unit* tests, only run programs"
  echo "   -p <PORT>   port on which the api should run"
  echo "   -a <NAME>   change name of compiled binary / process (experimental)"
  echo "   -t <SECS>   how many seconds to wait for scylla container to settle (experimental)"
}

trap 'cleanup' SIGINT SIGTERM

export SALT_ENCRYPTION_IV="ffA_1D6s^jf!6\$xx"
export SALT_ENCRYPTION_KEY='#a1aA3!h4a@ah3a4'
export SCYLLA_CASSANDRA_PASSWORD='cassandra'
export API_PORT=1313
export NO_OF_WORKERS=32
export SCYLLA_INET=172.17.0.2
export EXECUTABLE_NAME="api"

c_flag=''
s_flag=''
S_flag=''
u_flag=''
cargo_args=''
scylla_wait_time=''
while getopts 't:a:vchsSup:' flag; do
  case "${flag}" in
    c) c_flag='true' ;;
    s) s_flag='true' ;;
    t) scylla_wait_time="${OPTARG:-5}" ;;
    v) cargo_args="${cargo_args} --verbose";;
    a) export EXECUTABLE_NAME="${OPTARG:-api}";;
    S) S_flag='true' ;;
    u) u_flag='true' ;;
    p) export API_PORT="${OPTARG:-1313}" ;;
    h) print_usage
       exit 0 ;;
    *) print_usage
       exit 1 ;;
  esac
done

if [[ "${c_flag}" == "true" ]]; then
  cleanup
  exit 0
fi

if ! [[ "${S_flag}" == "true" ]]; then
  echo "======================= LAUNCHING SCYLLA ======================="
  ./launch-scylla-locally.sh "${scylla_wait_time}"
fi

echo "======================== COMPILING API ========================="
cargo build --release $cargo_args

echo "======================= LAUNCHING API ========================="
echo " * api is lauching on 127.0.0.1:${API_PORT} * "

if ! [[ "${EXECUTABLE_NAME}" == "api" ]]; then
  mv ./target/release/api "./target/release/${EXECUTABLE_NAME}"
  "./target/release/${EXECUTABLE_NAME}" > test_env.stdout 2> test_env.stderr &
else
  ./target/release/api > test_env.stdout 2> test_env.stderr &
fi

if [[ "${s_flag}" == "true" ]]; then
  echo "====================== SKIPPING TESTS ========================="
  exit 0
fi

if ! [[ "${u_flag}" == "true" ]]; then
  echo "================== LAUNCHING API UNIT TESTS ===================="
  cargo test $cargo_args 
fi

echo "================== LAUNCHING API E2E TESTS ===================="
./shadow/e2e.sh "http://localhost:${API_PORT}"
