#!/usr/bin/env bash

HOST=${1:-https://onlinedi.vision}
printf "HOST: %s\n" "${HOST}"
QA_USERNAME=qa_e2e_user-$(tr -dc A-Za-z0-9 </dev/urandom | head -c 13)

printf "QA_USERNAME: %s\n" "${QA_USERNAME}"

set -eo pipefail

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
    exit 1
  fi
  echo "PASSED"
}

function assert_match() {
  actual=${1}
  expected=${2}
  message=${3}
  if ! [[ "$expected" == $actual ]]; then
    echo ""
    echo " *** E2E TESTING FAILED: ${message}" >&2
    echo "      ${expected} DOES NOT EQUAL ${actual}" >&2
    echo "      EXPECTED |${expected}" >&2
    echo "      BUT GOT  |${actual}" >&2
    exit 1
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
    exit 2
  fi
``  echo "PASSED"
}

function eetest() {
  printf " Testing %s... " "${1}"
}

function get() {
  curl --silent "${HOST}${1}" 2>> .curl_stderr
}

function post() {
  curl --silent -X POST --header "Content-Type:application/json" -d "${1}" "${HOST}${2}"
}

eetest "/servers/{sid}/api/get_server_info" ""
get_server_info=$(get "/servers/1313/api/get_server_info"  | jq '.name')
assert '"division"' "${get_server_info}" "/servers/{sid}/api/get_server_info"

eetest "/api/version" ""
get_server_info=$(get "/api/version" )
assert_match v* "${get_server_info}" "/servers/{sid}/api/get_server_info"

eetest "/api/new_user"
token=$(post "{\"username\":\"${QA_USERNAME}\", \"password\":\"${QA_E2E_ACCOUNT_PASSWORD}\", \"email\":\"L\"}" "/api/new_user"| jq '.token')
assert_neq "null" "${token}" "/api/new_user"

eetest "/api/create_server"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"desc\":\"L\", \"name\":\"QA_TEST_SERVER\", \"img_url\":\"L\"}" "/api/create_server")
token=$(echo "$payload"  | jq '.token')
sid1=$(echo "$payload" | jq '.sid')
assert_neq "null" "${token}" "/api/create_server"

eetest "/api/get_user_servers"
user_servers_payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}}" "/api/get_user_servers")
sid=$(echo -e "${user_servers_payload}" | jq -r '.s_list[0]')
token=$(echo -e "${user_servers_payload}" | jq '.token')
assert_neq "null" "${token}" "/api/get_user_servers"

eetest "/api/create_server -- check api sent SID"
assert "$sid1" "\"$sid\"" "/api/create_server -- check api sent SID"

eetest "/servers/{sid}/api/get_server_info (part2) -- name" ""
get_server_info=$(get "/servers/${sid}/api/get_server_info"  | jq '.name')
assert '"QA_TEST_SERVER"' "${get_server_info}" "/servers/{sid}/api/get_server_info"

eetest "/servers/{sid}/api/get_server_info (part2) -- desc" ""
get_server_info=$(get "/servers/${sid}/api/get_server_info"  | jq '.desc')
assert '"L"' "${get_server_info}" "/servers/{sid}/api/get_server_info"

eetest "/servers/{sid}/api/get_server_info (part2) -- img_url" ""
get_server_info=$(get "/servers/${sid}/api/get_server_info"  | jq '.img_url')
assert '"L"' "${get_server_info}" "/servers/{sid}/api/get_server_info"

eetest "/servers/{sid}/api/create_channel" ""
token=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"channel_name\":\"main\"}" "/servers/${sid}/api/create_channel"  | jq '.token' )
assert_neq "null" "${token}" "/servers/${sid}/api/create_channel"

eetest "/servers/{sid}/api/get_channels" ""
main_channel=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}}" "/servers/${sid}/api/get_channels"  | jq -r '.c_list[1].channel_name' )
assert "main" "${main_channel}" "/servers/${sid}/api/get_channels"

eetest "/servers/{sid}/api/delete_server (${QA_USERNAME} is owner)" ""
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}}" "/servers/${sid}/api/delete_server"  )
assert 'Server deleted successfully' "${payload}" "/servers/{sid}/api/delete_server"

eetest "/servers/{sid}/api/delete_server (${QA_USERNAME} is _NOT_ owner)" ""
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}}" "/servers/1313/api/delete_server"  )
assert "You don't have permission to delete this server" "${payload}" "/servers/{sid}/api/delete_server"
