#!/usr/bin/env bash

# Build, push, and report metadata for one tagged release image.

set -euo pipefail

# Prefer Jenkins' manual parameter, falling back to the checked-out Git ref.

git_ref="${RELEASE_REF:-${GIT_BRANCH:-}}"
image_repository="${IMAGE_REPOSITORY:-registry.onlinedi.vision:5000/od-official-server}"
report_file="${IMAGE_REPORT:-image-size-report.md}"

# Prevent branches and malformed refs from publishing release images.

if [[ ! "$git_ref" =~ ^refs/tags/([^/]+)$ ]]; then
    printf 'Ref must have the form refs/tags/VERSION; received: %s\n' \
        "${git_ref:-<empty>}" >&2
    exit 2
fi

release_version="${BASH_REMATCH[1]}"
image_tag="v${release_version}"

# Ensure the generated value is a valid Docker image tag.

if [[ ! "$image_tag" =~ ^[A-Za-z0-9_][A-Za-z0-9_.-]{0,127}$ ]]; then
    printf 'Invalid container tag: %s\n' "$image_tag" >&2
    exit 2
fi

image_ref="${image_repository}:${image_tag}"

# docker-bake.hcl expects this variable.
export GIT_BRANCH="$git_ref"

# Build and load one image into Docker.
docker buildx bake release \
    --set 'release.output=type=docker' \
    --set "release.tags=${image_ref}"

# Push the exact image that was just built.
docker push "$image_ref"

size_bytes="$(docker image inspect --format '{{.Size}}' "$image_ref")"

# Read local image metadata after the image has been pushed.

if [[ ! "$size_bytes" =~ ^[0-9]+$ ]]; then
    printf 'Docker returned an invalid image size: %s\n' "$size_bytes" >&2
    exit 1
fi

size_human="$(awk -v bytes="$size_bytes" 'BEGIN {
    split("B KiB MiB GiB TiB", units, " ");
    size = bytes;
    unit = 1;

    while (size >= 1024 && unit < 5) {
        size /= 1024;
        unit++;
    }

    if (unit == 1)
        printf "%d %s", size, units[unit];
    else
        printf "%.2f %s", size, units[unit];
}')"

image_id="$(docker image inspect --format '{{.Id}}' "$image_ref")"
repo_digests="$(
    docker image inspect \
        --format '{{range .RepoDigests}}{{println .}}{{end}}' \
        "$image_ref"
)"

repo_digest="unavailable"

while IFS= read -r candidate; do
    if [[ -n "$candidate" ]]; then
        repo_digest="$candidate"
        break
    fi
done <<< "$repo_digests"

git_commit="${GIT_COMMIT:-unknown}"

{
    printf '# Container image report\n\n'
    printf '| Field | Value |\n'
    printf '| --- | --- |\n'
    printf '| Image | `%s` |\n' "$image_ref"
    printf '| Repository digest | `%s` |\n' "$repo_digest"
    printf '| Image ID | `%s` |\n' "$image_id"
    printf '| Local image size | %s (%s bytes) |\n' \
        "$size_human" "$size_bytes"
    printf '| Git commit | `%s` |\n' "$git_commit"
    printf '| Generated at | %s |\n' \
        "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
} > "$report_file"

printf 'Published %s\n' "$image_ref"
printf 'Image size: %s (%s bytes)\n' "$size_human" "$size_bytes"