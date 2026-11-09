#!/usr/bin/env bash
# Prepare a signed release locally. Hosting and catalogue activation are separate.
set -euo pipefail
if [[ "$#" != 4 ]]; then
  printf 'Usage: %s PACKAGE_DIRECTORY SIGNING_FINGERPRINT HTTPS_REPOSITORY_URL NEW_OUTPUT_DIRECTORY\n' "$0" >&2
  exit 2
fi
package=$(realpath -- "$1")
fingerprint=${2^^}
url=${3%/}
output=$(realpath -m -- "$4")
[[ "$fingerprint" =~ ^([A-F0-9]{40}|[A-F0-9]{64})$ ]] || { printf 'Use the full release-signing key fingerprint.\n' >&2; exit 2; }
[[ "$url" =~ ^https://[A-Za-z0-9][A-Za-z0-9.-]*(:[0-9]+)?(/[A-Za-z0-9._~%/-]*)?$ ]] || { printf 'Use an HTTPS repository URL without credentials, query, or fragment.\n' >&2; exit 2; }
test ! -e "$output" || { printf 'The output directory already exists. Choose a new directory.\n' >&2; exit 2; }
for tool in flatpak ostree gpg sha256sum base64; do command -v "$tool" >/dev/null; done
# Only accept the normal package artifact's four literal provenance files.
test "$(wc -l < "$package/checksums.txt")" -eq 4
test "$(grep -Ec '^[a-f0-9]{64}  (alo-studio-0\.1\.0\.flatpak|source\.txt|features\.txt|sdk-versions\.txt)$' "$package/checksums.txt")" -eq 4
for name in alo-studio-0.1.0.flatpak source.txt features.txt sdk-versions.txt; do
  test "$(grep -Fc "  $name" "$package/checksums.txt")" -eq 1
done
(cd "$package"; sha256sum --strict --check checksums.txt)
test "$(cat "$package/features.txt")" = 'normal; verification disabled'
source=$(cat "$package/source.txt")
[[ "$source" =~ ^[a-f0-9]{40}$ ]]
gpg --batch --list-secret-keys "$fingerprint" >/dev/null 2>&1 || { printf 'The specified signing key is unavailable.\n' >&2; exit 1; }
mkdir -p -- "$output"
repository="$output/repository"
ostree init --repo="$repository" --mode=archive-z2
flatpak build-import-bundle "$repository" "$package/alo-studio-0.1.0.flatpak"
mapfile -t refs < <(ostree --repo="$repository" refs)
[[ "${#refs[@]}" == 1 && "${refs[0]}" =~ ^app/org\.aloworld\.Studio/([A-Za-z0-9_-]+)/([A-Za-z0-9._-]+)$ ]] || { printf 'The bundle does not contain exactly one Studio application ref.\n' >&2; exit 1; }
arch=${BASH_REMATCH[1]}
branch=${BASH_REMATCH[2]}
ref=${refs[0]}
flatpak build-sign --arch="$arch" --gpg-sign="$fingerprint" "$repository" org.aloworld.Studio "$branch"
gpg --batch --export-options export-minimal --export "$fingerprint" > "$output/release-key.gpg"
test -s "$output/release-key.gpg"
flatpak build-update-repo --gpg-sign="$fingerprint" --gpg-import="$output/release-key.gpg" --title='alo applications' --default-branch="$branch" "$repository"
key=$(base64 -w0 "$output/release-key.gpg")
cat > "$output/alo-applications.flatpakrepo" <<EOF
[Flatpak Repo]
Title=alo applications
Url=$url
DefaultBranch=$branch
GPGKey=$key
EOF
cat > "$output/alo-studio.flatpakref" <<EOF
[Flatpak Ref]
Name=org.aloworld.Studio
Title=alo Studio
Branch=$branch
Url=$url
IsRuntime=false
GPGKey=$key
RuntimeRepo=https://flathub.org/repo/flathub.flatpakrepo
EOF
cp -- "$package/source.txt" "$package/features.txt" "$package/sdk-versions.txt" "$output/"
commit=$(ostree --repo="$repository" rev-parse "$ref")
printf 'source=%s\nref=%s\ncommit=%s\nsigning_fingerprint=%s\nrepository_url=%s\n' "$source" "$ref" "$commit" "$fingerprint" "$url" > "$output/release-provenance.txt"
(cd "$output"
  files=(release-key.gpg alo-applications.flatpakrepo alo-studio.flatpakref source.txt features.txt sdk-versions.txt release-provenance.txt repository/summary repository/summary.sig)
  if [[ -f repository/summary.idx ]]; then
    test -f repository/summary.idx.sig
    files+=(repository/summary.idx repository/summary.idx.sig)
  fi
  sha256sum "${files[@]}" > checksums.txt
)
printf 'Signed repository prepared: %s\nApplication ref: %s\nCommit: %s\n' "$repository" "$ref" "$commit"
