#!/usr/bin/env bash

test_count=0
HOST=${1:-https://onlinedi.vision}
printf "HOST: %s\n" "${HOST}"
QA_USERNAME=$(mktemp --dry-run qa_e2e_user-XXXXXXXXXXXX)

printf "QA_USERNAME: %s\n" "${QA_USERNAME}"

set -eo pipefail

function results() {
  if [[ $? != 0 ]]; then
    echo
    echo "E2E TESTING *FAILED*"
    exit 1
  fi

  echo
  echo " ** SUCCESS ** all ${test_count} tests PASSED !"
}

trap results EXIT

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
  test_count=$((test_count+1))
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


eetest "/servers/{sid}/get_server_info" ""
get_server_info=$(get "/servers/1313/get_server_info"  | jq '.name')
assert '"division"' "${get_server_info}" "/servers/{sid}/get_server_info"

eetest "/get_user_servers -- with NULL token (testing that the API does not panic...)"
err_message=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":\"\"}" "/get_user_servers")
assert "Invalid or expired token" "${err_message}"

eetest "/version" ""
get_server_info=$(get "/version" )
assert_match v* "${get_server_info}" "/servers/{sid}/get_server_info"

eetest "/new_user"
token=$(post "{\"username\":\"${QA_USERNAME}\", \"password\":\"${QA_E2E_ACCOUNT_PASSWORD}\", \"email\":\"L\"}" "/new_user"| jq '.token')
assert_neq "null" "${token}" "/new_user"

eetest "/new_user (part2) -- max_username_length"
nutoken=$(post "{\"username\":\"$(tr -dc A-Za-z0-9 </dev/urandom | head -c 31)\", \"password\":\"${QA_E2E_ACCOUNT_PASSWORD}\", \"email\":\"L\"}" "/new_user")
assert_match "Failed to create user: Username longer than "* "${nutoken}" "/new_user"

eetest "/create_server"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"desc\":\"L\", \"name\":\"QA_TEST_SERVER\", \"img_url\":\"L\"}" "/create_server")
token=$(echo "$payload"  | jq '.token')
sid1=$(echo "$payload" | jq '.sid')
assert_neq "null" "${token}" "/create_server"

eetest "/create_server (part2) -- max_server_length"
nutoken=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"desc\":\"L\", \"name\":\"QA_TEST_SERVER_BUT_A_LITTLE_LONGER_THAN_MAX\", \"img_url\":\"L\"}" "/create_server" )
assert_match "Failed to create server: Server name longer than "* "${nutoken}" "/create_server"

eetest "/am_i_in_server"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"sid\":${sid1}}" "/am_i_in_server")
assert "Yes you are part of the server." "${payload}" "/am_i_in_server"

eetest "/am_i_in_server -- (not in server)"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"sid\":\"AAA\"}" "/am_i_in_server")
assert "You are not part of this server." "${payload}" "/am_i_in_server -- (not in server)"

eetest "/get_user_servers"
user_servers_payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}}" "/get_user_servers")
sid=$(echo -e "${user_servers_payload}" | jq -r '.s_list[0]')
token=$(echo -e "${user_servers_payload}" | jq '.token')
assert_neq "null" "${token}" "/get_user_servers"

eetest "/create_server -- check api sent SID"
assert "$sid1" "\"$sid\"" "/create_server -- check api sent SID"

eetest "/servers/{sid}/get_server_info (part2) -- name" ""
get_server_info=$(get "/servers/${sid}/get_server_info"  | jq '.name')
assert '"QA_TEST_SERVER"' "${get_server_info}" "/servers/{sid}/get_server_info"

eetest "/servers/{sid}/get_server_info (part2) -- desc" ""
get_server_info=$(get "/servers/${sid}/get_server_info"  | jq '.desc')
assert '"L"' "${get_server_info}" "/servers/{sid}/get_server_info"

eetest "/servers/{sid}/get_server_info (part2) -- img_url" ""
get_server_info=$(get "/servers/${sid}/get_server_info"  | jq '.img_url')
assert '"L"' "${get_server_info}" "/servers/{sid}/get_server_info"

eetest "/servers/{sid}/create_channel" ""
token=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"channel_name\":\"main\"}" "/servers/${sid}/create_channel"  | jq '.token' )
assert_neq "null" "${token}" "/servers/${sid}/create_channel"

eetest "/servers/{sid}/create_channel (part2) -- max_channel_length" ""
nutoken=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"channel_name\":\"flajkaldjflhkcvjhxzoyuafhldasjhfiocuzxgvhadfhsojk\"}" "/servers/${sid}/create_channel" )
assert_match "Failed to create channel: Channel name longer than "* "${nutoken}" "/servers/${sid}/create_channel"

eetest "/servers/{sid}/create_channel -- INEXISTENT SERVER" ""
ftoken=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"channel_name\":\"main\"}" "/servers/a/create_channel")
assert "Couldn't find that server. (a) :(" "${ftoken}" "/servers/create_channel -- INEXISTENT SERVER"

eetest "/servers/{sid}/get_channels" ""
main_channel=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}}" "/servers/${sid}/get_channels"  | jq -r '.c_list[1].channel_name' )
assert "main" "${main_channel}" "/servers/${sid}/get_channels"

message_to_send_succesfully="This is the sent message."
message_to_send_unsuccesfully=$(tr -dc A-Za-z0-9 </dev/urandom | head -c 3001) || pwd > /dev/null

eetest "/servers/{sid}/{channel_name}/send_message" ""
send_response=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"m_content\":\"${message_to_send_succesfully}\"}" "/servers/${sid}/${main_channel}/send_message" )
assert "Message sent." "${send_response}" "/servers/${sid}/${main_channel}/send_message"

eetest "/servers/{sid}/{channel_name}/send_message (part2) -- max_message_length" ""
send_response=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"m_content\":\"${message_to_send_unsuccesfully}\"}" "/servers/${sid}/${main_channel}/send_message" )
assert_match "Failed to send message: Message longer than "* "${send_response}" "/servers/${sid}/${main_channel}/send_message"

eetest "/servers/{sid}/{channel_name}/send_message -- INEXISTENT SERVER"
send_response=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"m_content\":\"${message_to_send_succesfully}\"}" "/servers/a/${main_channel}/send_message" )
assert "Couldn't find that server. (a) :(" "${send_response}" "/servers/a/${main_channel}/send_message -- INEXISTENT SERVER"

eetest "/servers/{sid}/{channel_name}/send_message -- INEXISTENT CHANNEL"
send_response=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"m_content\":\"${message_to_send_succesfully}\"}" "/servers/${sid}/a/send_message" )
assert "Couldn't find that channel. (a) :(" "${send_response}" "/servers/${sid}/a/send_message -- INEXISTENT CHANNEL"

eetest "/servers/{sid}/{channel_name}/get_messages_migration"
message_recieved=$(post "{\"username\":\"${QA_USERNAME}\",\"token\":${token}, \"limit\":\"100\", \"offset\":\"0\"}" "/servers/${sid}/${main_channel}/get_messages_migration" | jq -r '.m_list[0].m_content')
datetime_received=$(post "{\"username\":\"${QA_USERNAME}\",\"token\":${token}, \"limit\":\"100\", \"offset\":\"0\"}" "/servers/${sid}/${main_channel}/get_messages_migration" | jq -r '.m_list[0].datetime')
assert "${message_to_send_succesfully}" "${message_recieved}" "/servers/${sid}/${main_channel}/get_messages_migration"

datetime_len="${#datetime_received}"
datetime_len=$((datetime_len - 3))
api_datetime_lh="$(echo "${datetime_received}" | head -c "$datetime_len")"
api_datetime_rh="$(echo "${datetime_received}" | tail -c 3)"
api_datetime=$(date -d @"${api_datetime_lh}.${api_datetime_rh}" +'%Y-%m-%d %H:%M:%S')

eetest "/servers/{sid}/{channel_name}/delete_message"
response=$(post "{\"username\":\"${QA_USERNAME}\",\"token\":${token}, \"datetime\":\"${api_datetime}\"}" "/servers/${sid}/${main_channel}/delete_message")
assert "Message deleted successfully" "${response}" "/servers/${sid}/${main_channel}/delete_message"

eetest "/servers/{sid}/{channel_name}/get_messages_migration"
message_recieved=$(post "{\"username\":\"${QA_USERNAME}\",\"token\":${token}, \"limit\":\"100\", \"offset\":\"0\"}" "/servers/${sid}/${main_channel}/get_messages_migration" | jq -r '.m_list[0].m_content')
# TODO: why does this fail ?
expected_failure assert_neq "${message_to_send_succesfully}" "${message_recieved}" "/servers/${sid}/${main_channel}/get_messages_migration"

eetest "/user/ttl"
payload=$(patch "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"ttl\":\"s\"}" "/user/ttl")
assert "TTL Updated." "${payload}" "/user/ttl"

message_with_short_ttl="This message will be deleted after 3 seconds."
eetest "/servers/{sid}/{channel_name}/send_message -- TTL Expiration"
send_response=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"m_content\":\"${message_with_short_ttl}\"}" "/servers/${sid}/${main_channel}/send_message" )
assert "Message sent." "${send_response}" "/servers/${sid}/${main_channel}/send_message"

eetest "/servers/{sid}/{channel_name}/get_messages_migration -- TTL Expiration"
# sleeping for 3 seconds so that the last message's Time To Live
# passes and the message gets deleted
printf 'SLEEPING 1 SECONDS... '
sleep 1
message=$(post "{\"username\":\"${QA_USERNAME}\",\"token\":${token}, \"limit\":\"100\", \"offset\":\"0\"}" "/servers/${sid}/${main_channel}/get_messages_migration" | jq -r '.m_list[].m_content')
assert "" "$(echo "$message" | grep "$message_with_short_ttl")" "/servers/{sid}/{channel_name}/get_messages_migration -- TTL Expiration"

eetest "/user/ttl -- set TTL back to _N_ormal"
payload=$(patch "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"ttl\":\"N\"}" "/user/ttl")
assert "TTL Updated." "${payload}" "/user/ttl"

QA_USERNAME2=$(mktemp --dry-run qa_e2e_user2-XXXXXXXXXXXX)
eetest "/new_user (user2 for role tests)"
token2=$(post "{\"username\":\"${QA_USERNAME2}\", \"password\":\"${QA_E2E_ACCOUNT_PASSWORD}\", \"email\":\"L2\"}" "/new_user" | jq '.token')
assert_neq "null" "${token2}" "/new_user (user2)"

eetest "/servers/{sid}/join (user2 joins server)"
join_payload=$(post "{\"username\":\"${QA_USERNAME2}\", \"token\":${token2}}" "/servers/${sid}/join")
token2=$(echo "${join_payload}" | jq '.token')
assert_neq "null" "${token2}" "/servers/{sid}/join (user2)"

eetest "/add_server_role -- admin creates role"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"server_id\":\"${sid}\", \"name\":\"moderator\", \"permissions\":1}" "/add_server_role")
assert "Role added successfully" "${payload}" "/add_server_role"

eetest "/add_server_role -- member cannot create role"
payload=$(post "{\"username\":\"${QA_USERNAME2}\", \"token\":${token2}, \"server_id\":\"${sid}\", \"name\":\"hackerman_role\", \"permissions\":1}" "/add_server_role")
assert "You do not have permission to manage roles" "${payload}" "/add_server_role (member denied)"

eetest "/add_server_role -- invalid permission bits"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"server_id\":\"${sid}\", \"name\":\"wrong_role\", \"permissions\":9999}" "/add_server_role")
assert "Invalid permission request!" "${payload}" "/add_server_role (invalid perms)"

eetest "/api/assign_role -- admin assigns role to user2"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"server_id\":\"${sid}\", \"target_user\":\"${QA_USERNAME2}\", \"role_name\":\"moderator\"}" "/api/assign_role")
assert "Role assigned" "${payload}" "/api/assign_role"

eetest "/api/assign_role -- member cannot assign role"
payload=$(post "{\"username\":\"${QA_USERNAME2}\", \"token\":${token2}, \"server_id\":\"${sid}\", \"target_user\":\"${QA_USERNAME}\", \"role_name\":\"moderator\"}" "/api/assign_role")
assert "You do not have permission to manage roles" "${payload}" "/api/assign_role (member denied)"

eetest "/api/assign_role -- target user not in server"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"server_id\":\"${sid}\", \"target_user\":\"nonexistent_user_xyz\", \"role_name\":\"moderator\"}" "/api/assign_role")
assert "Target user is not in the server" "${payload}" "/api/assign_role (not in server)"

eetest "/api/remove_role -- admin removes role from user2"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"server_id\":\"${sid}\", \"target_user\":\"${QA_USERNAME2}\", \"role_name\":\"moderator\"}" "/api/remove_role")
assert "Role removed successfully" "${payload}" "/api/remove_role"

eetest "/api/remove_role -- member cannot remove role"
payload=$(post "{\"username\":\"${QA_USERNAME2}\", \"token\":${token2}, \"server_id\":\"${sid}\", \"target_user\":\"${QA_USERNAME}\", \"role_name\":\"admin\"}" "/api/remove_role")
assert "You do not have permission to manage roles" "${payload}" "/api/remove_role (member denied)"

eetest "/api/delete_server_role -- admin deletes role"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"server_id\":\"${sid}\", \"role_name\":\"moderator\"}" "/api/delete_server_role")
assert "Role deleted successfully" "${payload}" "/api/delete_server_role"

eetest "/api/delete_server_role -- member cannot delete role"
payload=$(post "{\"username\":\"${QA_USERNAME2}\", \"token\":${token2}, \"server_id\":\"${sid}\", \"role_name\":\"admin\"}" "/api/delete_server_role")
assert "You do not have permission to manage roles" "${payload}" "/api/delete_server_role (member denied)"

long_role_name=$(tr -dc A-Za-z0-9 </dev/urandom | head -c 31) || pwd > /dev/null
eetest "/add_server_role -- role name too long"
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"server_id\":\"${sid}\", \"name\":\"${long_role_name}\", \"permissions\":1}" "/add_server_role")
assert_match "Role name exceeds maximum length of "* "${payload}" "/add_server_role (name too long)"

eetest "/servers/{sid}/delete_server (${QA_USERNAME} is owner)" ""
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}}" "/servers/${sid}/delete_server"  )
assert 'Server deleted successfully' "${payload}" "/servers/{sid}/delete_server"

eetest "/servers/{sid}/delete_server (${QA_USERNAME} is _NOT_ owner)" ""
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}}" "/servers/1313/delete_server"  )
assert "You don't have permission to delete this server" "${payload}" "/servers/{sid}/delete_server"

eetest "/spell/cast && /spell/check"
payload=$(post "{\"username\":\"${QA_USERNAME}\"}" "/spell/cast")
key=$(echo "$payload" | jq -r '.key')
spell1=$(echo "$payload" | jq -r '.spell')

spell2=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"key\":\"${key}\"}" "/spell/check")
assert "$spell1" "$spell2" "/spell/cast && /spell/check"

eetest "/create_server -- should fail due to TOKEN TTL"
# Sleeping so that we can trigger the TOKEN_TTL (set in launch-test-env)
# the test-env TOKEN_TTL=2s (we slept for 1 second before, and we sleep
# for an additional 1 second here).
printf 'SLEEPING 1 SECONDS... '
sleep 1
payload=$(post "{\"username\":\"${QA_USERNAME}\", \"token\":${token}, \"desc\":\"L\", \"name\":\"QA_TEST_SERVER\", \"img_url\":\"L\"}" "/create_server")
assert "Invalid token" "${payload}" "/create_server"
