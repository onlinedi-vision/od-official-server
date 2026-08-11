#!/usr/bin/env bash

# Verify publishing behavior safely by replacing Docker with the local fake.

set -euo pipefail

# Keep fake commands, logs, and reports isolated from the repository.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
test_directory="$(mktemp -d)"
trap 'rm -rf "$test_directory"' EXIT

fail() {
    printf 'FAIL: %s\n' "$1" >&2
    exit 1
}

# Place the fake first in PATH under the name expected by the production script.

cp "$repo_root/ci/tests/fake-docker" "$test_directory/docker"
chmod +x "$test_directory/docker"

docker_log="$test_directory/docker.log"
report_file="$test_directory/image-size-report.md"

# Happy path: a valid release must build once, push once, and create a report.

FAKE_DOCKER_LOG="$docker_log" \
PATH="$test_directory:$PATH" \
RELEASE_REF='refs/tags/1.2.3' \
GIT_COMMIT='0123456789abcdef' \
IMAGE_REPORT="$report_file" \
    "$repo_root/ci/publish-image.sh"

# Confirm that publishing did not rebuild or push the image more than once.

build_count="$(grep -c '^buildx bake release ' "$docker_log")"
push_count="$(
    grep -c \
        '^push registry.onlinedi.vision:5000/od-official-server:v1.2.3$' \
        "$docker_log"
)"

[[ "$build_count" == "1" ]] ||
    fail 'expected exactly one image build'

[[ "$push_count" == "1" ]] ||
    fail 'expected exactly one image push'

# Confirm that the archived report contains the expected size and digest.

grep -q '12.00 MiB (12582912 bytes)' "$report_file" ||
    fail 'report is missing the image size'

grep -q 'sha256:digest' "$report_file" ||
    fail 'report is missing the pushed digest'


# Safety case: branch refs must be rejected before Docker is called.

: > "$docker_log"

if FAKE_DOCKER_LOG="$docker_log" \
    PATH="$test_directory:$PATH" \
    RELEASE_REF='refs/heads/main' \
        "$repo_root/ci/publish-image.sh"; then
    fail 'branch references must not be publishable'
fi

[[ ! -s "$docker_log" ]] ||
    fail 'invalid refs must be rejected before calling Docker'

# Failure case: a failed build must stop execution before the push.

: > "$docker_log"

if FAKE_DOCKER_LOG="$docker_log" \
    FAKE_DOCKER_FAIL_BUILD=1 \
    PATH="$test_directory:$PATH" \
    RELEASE_REF='refs/tags/2.0.0' \
        "$repo_root/ci/publish-image.sh"; then
    fail 'a failed build must fail the publishing command'
fi

failed_pushes="$(grep -c '^push ' "$docker_log" || true)"

[[ "$failed_pushes" == "0" ]] ||
    fail 'the script pushed an image after a failed build'

printf 'PASS: image publishing and report generation\n'