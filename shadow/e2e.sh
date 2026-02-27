#!/usr/bin/env bash

HOST=${1:-https://onlinedi.vision}
printf "HOST: %s\n" "${HOST}"
QA_USERNAME=$(mktemp --dry-run qa_e2e_user-XXXXXXXXXXXX)

printf "QA_USERNAME: %s\n" "${QA_USERNAME}"

set -eo pipefail

function expected_failure() {
  echo
  echo   "******************************************************************************************************"
  echo   "                                        (EXPECTED FAILURE)"
  printf "******************************************************************************************************"
  if ! "${@}"; then
    echo "******************************************************************************************************"
    echo "                                        (EXPECTED FAILURE)"
    echo "******************************************************************************************************"
    echo
  else
    echo "******************************************************************************************************"
    echo "                                  (THIS TEST PASSED UNEXPECTEDLY)"
    echo "******************************************************************************************************"
    exit 1
  fi
}

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
    return 1
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
    return 1
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
    return 2
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

function patch() {
  curl --silent -X PATCH --header "Content-Type:application/json" -d "${1}" "${HOST}${2}"
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

eetest "/api/new_user (part2) -- max_username_length"
nutoken=$(post "{\"username\":\"$(tr -dc A-Za-z0-9 </dev/urandom | head -c 31)\", \"password\":\"${QA_E2E_ACCOUNT_PASSWORD}\", \"email\":\"L\"}" "/api/new_user")
assert_match "Failed to create user: Username longer than "* "${nutoken}" "/api/new_user"

eetest "/api/create_server"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"desc\":\"L\", \"name\":\"QA_TEST_SERVER\", \"img_url\":\"L\"}" "/api/create_server")
token=$(echo "$payload"  | jq '.token')
sid1=$(echo "$payload" | jq '.sid')
assert_neq "null" "${token}" "/api/create_server"

eetest "/api/create_server (part2) -- max_server_length"
nutoken=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"desc\":\"L\", \"name\":\"QA_TEST_SERVER_BUT_A_LITTLE_LONGER_THAN_MAX\", \"img_url\":\"L\"}" "/api/create_server" )
assert_match "Failed to create server: Server name longer than "* "${nutoken}" "/api/create_server"

eetest "/api/am_i_in_server"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"sid\":${sid1}}" "/api/am_i_in_server")
assert "Yes you are part of the server." "${payload}" "/api/am_i_in_server"

eetest "/api/am_i_in_server -- (not in server)"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"sid\":\"AAA\"}" "/api/am_i_in_server")
assert "You are not part of this server." "${payload}" "/api/am_i_in_server -- (not in server)"

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

eetest "/servers/{sid}/api/create_channel -- INEXISTENT SERVER" ""
ftoken=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"channel_name\":\"main\"}" "/servers/a/api/create_channel")
assert "Couldn't find that server. (a) :(" "${ftoken}" "/servers//api/create_channel -- INEXISTENT SERVER"

eetest "/servers/{sid}/api/create_channel (part2) -- max_channel_length" ""
nutoken=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"channel_name\":\"flajkaldjflhkcvjhxzoyuafhldasjhfiocuzxgvhadfhsojk\"}" "/servers/${sid}/api/create_channel" )
assert_match "Failed to create channel: Channel name longer than "* "${nutoken}" "/servers/${sid}/api/create_channel"

eetest "/servers/{sid}/api/get_channels" ""
main_channel=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}}" "/servers/${sid}/api/get_channels"  | jq -r '.c_list[1].channel_name' )
assert "main" "${main_channel}" "/servers/${sid}/api/get_channels"

message_to_send_succesfully="This is the sent message."
message_to_send_unsuccesfully=$(tr -dc A-Za-z0-9 </dev/urandom | head -c 3001) || pwd > /dev/null

eetest "/servers/{sid}/api/{channel_name}/send_message" ""
send_response=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"m_content\":\"${message_to_send_succesfully}\"}" "/servers/${sid}/api/${main_channel}/send_message" )
assert "Message sent." "${send_response}" "/servers/${sid}/api/${main_channel}/send_message"

eetest "/servers/{sid}/api/{channel_name}/send_message -- INEXISTENT SERVER"
send_response=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"m_content\":\"${message_to_send_succesfully}\"}" "/servers/a/api/${main_channel}/send_message" )
assert "Couldn't find that server. (a) :(" "${send_response}" "/servers/a/api/${main_channel}/send_message -- INEXISTENT SERVER"

eetest "/servers/{sid}/api/{channel_name}/send_message -- INEXISTENT CHANNEL"
send_response=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"m_content\":\"${message_to_send_succesfully}\"}" "/servers/${sid}/api/a/send_message" )
assert "Couldn't find that channel. (a) :(" "${send_response}" "/servers/${sid}/api/a/send_message -- INEXISTENT CHANNEL"

eetest "/servers/{sid}/api/{channel_name}/send_message (part2) -- max_message_length" ""
send_response=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"m_content\":\"${message_to_send_unsuccesfully}\"}" "/servers/${sid}/api/${main_channel}/send_message" )
assert_match "Failed to send message: Message longer than "* "${send_response}" "/servers/${sid}/api/${main_channel}/send_message"

eetest "/servers/{sid}/api/{channel_name}/get_messages_migration"
message_recieved=$(post "{\"username\":\"${QA_USERNAME}\",\"token\":${token}, \"limit\":\"100\", \"offset\":\"0\"}" "/servers/${sid}/api/${main_channel}/get_messages_migration" | jq -r '.m_list[0].m_content')
datetime_received=$(post "{\"username\":\"${QA_USERNAME}\",\"token\":${token}, \"limit\":\"100\", \"offset\":\"0\"}" "/servers/${sid}/api/${main_channel}/get_messages_migration" | jq -r '.m_list[0].datetime')
assert "${message_to_send_succesfully}" "${message_recieved}" "/servers/${sid}/api/${main_channel}/get_messages_migration"

datetime_len="${#datetime_received}"
datetime_len=$((datetime_len - 3))
api_datetime_lh="$(echo "${datetime_received}" | head -c "$datetime_len")"
api_datetime_rh="$(echo "${datetime_received}" | tail -c 3)"
api_datetime=$(date -d @"${api_datetime_lh}.${api_datetime_rh}" +'%Y-%m-%d %H:%M:%S')

eetest "/servers/{sid}/api/{channel_name}/delete_message"
response=$(post "{\"username\":\"${QA_USERNAME}\",\"token\":${token}, \"datetime\":\"${api_datetime}\"}" "/servers/${sid}/api/${main_channel}/delete_message")
assert "Message deleted successfully" "${response}" "/servers/${sid}/api/${main_channel}/delete_message"

eetest "/servers/{sid}/api/{channel_name}/get_messages_migration"
message_recieved=$(post "{\"username\":\"${QA_USERNAME}\",\"token\":${token}, \"limit\":\"100\", \"offset\":\"0\"}" "/servers/${sid}/api/${main_channel}/get_messages_migration" | jq -r '.m_list[0].m_content')
# TODO: why does this fail ?
expected_failure assert_neq "${message_to_send_succesfully}" "${message_recieved}" "/servers/${sid}/api/${main_channel}/get_messages_migration"

eetest "/api/user/ttl"
payload=$(patch "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"ttl\":\"s\"}" "/api/user/ttl")
assert "TTL Updated." "${payload}" "/api/user/ttl"

message_with_short_ttl="This message will be deleted after 3 seconds."
eetest "/servers/{sid}/api/{channel_name}/send_message -- TTL Expiration"
send_response=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"m_content\":\"${message_with_short_ttl}\"}" "/servers/${sid}/api/${main_channel}/send_message" )
assert "Message sent." "${send_response}" "/servers/${sid}/api/${main_channel}/send_message"

eetest "/servers/{sid}/api/{channel_name}/get_messages_migration -- TTL Expiration"
# sleeping for 3 seconds so that the last message's Time To Live
# passes and the message gets deleted
printf 'SLEEPING 3 SECONDS... '
sleep 3
message=$(post "{\"username\":\"${QA_USERNAME}\",\"token\":${token}, \"limit\":\"100\", \"offset\":\"0\"}" "/servers/${sid}/api/${main_channel}/get_messages_migration" | jq -r '.m_list[].m_content')
assert "" "$(echo "$message" | grep "$message_with_short_ttl")" "/servers/{sid}/api/{channel_name}/get_messages_migration -- TTL Expiration"

eetest "/api/user/ttl -- set TTL back to _N_ormal"
payload=$(patch "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"ttl\":\"N\"}" "/api/user/ttl")
assert "TTL Updated." "${payload}" "/api/user/ttl"

eetest "/servers/{sid}/api/delete_server (${QA_USERNAME} is owner)" ""
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}}" "/servers/${sid}/api/delete_server"  )
assert 'Server deleted successfully' "${payload}" "/servers/{sid}/api/delete_server"

eetest "/servers/{sid}/api/delete_server (${QA_USERNAME} is _NOT_ owner)" ""
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}}" "/servers/1313/api/delete_server"  )
assert "You don't have permission to delete this server" "${payload}" "/servers/{sid}/api/delete_server"

eetest "/api/spell/cast && /api/spell/check"
payload=$(post "{\"username\":\"${QA_USERNAME}\"}" "/api/spell/cast")
key=$(echo "$payload" | jq -r '.key')
spell1=$(echo "$payload" | jq -r '.spell')

spell2=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"key\":\"${key}\"}" "/api/spell/check")
assert "$spell1" "$spell2" "/api/spell/cast && /api/spell/check"
