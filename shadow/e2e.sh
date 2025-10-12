#/usr/bin/env bash



HOST=${1:-https://onlinedi.vision}
CURL_POST_HEADER='--header "Content-Type: application/json"   --request POST'

function assert() {
  expected=${1}
  actual=${2}
  message=${3}
  if [[ "$expected" != "$actual" ]]; then
    echo ""
    echo " *** E2E TESTING FAILED: ${message}" >&2
    echo "      ${expected} DOES NOT EQUAL ${actual}" >&2
    echo "      EXPECTED |${expected}" >&2
    echo "      BUT GOT  |${actual}" >&2
    exit -1
  fi
  echo "PASSED"
}

function assert_neq() {
  expected=${1}
  actual=${2}
  message=${3}
  if [[ "$expected" == "$actual" ]]; then
    echo ""
    echo " *** E2E TESTING FAILED: ${message}" >&2
    echo "      ${expected} DOES EQUAL ${actual} (THEY SHOULD BE DISTINCT)" >&2
    echo "      EXPECTED                        |${expected}" >&2
    echo "      WHICH SHOULD BE DIFFERENT FROM  |${actual}" >&2
    exit -1
  fi
  echo "PASSED"
}

function eetest() {
  printf " Testing ${1}... "
}

eetest "/servers/{sid}/api/get_server_info" ""
get_server_info=$(curl ${HOST}/servers/1313/api/get_server_info 2> /dev/null | jq '.name')
assert '"division"' "${get_server_info}" "/servers/{sid}/api/get_server_info"

eetest "/api/new_user"
token=$(curl ${CURL_POST_HEADER} "{\"username\":\"qa_e2e_account\", \"password\":\"${QA_E2E_ACCOUNT_PASSWORD}\", \"email\":\"L\"}" "${HOST}/api/new_user"| jq '.token')
assert_neq "null" "${token}" "/api/new_user"

