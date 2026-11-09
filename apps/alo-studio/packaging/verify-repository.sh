#!/usr/bin/env bash
# Temporary keys and client installations are test fixtures, never releases.
set -euo pipefail
package=${1:?Pass the normal package artifact directory}
packaging=$(cd -- "$(dirname -- "$0")" && pwd)
scratch=$(mktemp -d)
cleanup() { gpgconf --homedir "$scratch/keys" --kill all 2>/dev/null || true; rm -rf -- "$scratch"; }
trap cleanup EXIT
export GNUPGHOME="$scratch/keys"
mkdir -m 700 "$GNUPGHOME"
gpg --batch --pinentry-mode loopback --passphrase '' --quick-generate-key 'Studio repository verification only' ed25519 sign 1d
fingerprint=$(gpg --batch --with-colons --list-secret-keys | awk -F: '$1 == "fpr" { print $10; exit }')
bash "$packaging/export-repository.sh" "$package" "$fingerprint" https://studio.invalid/repository "$scratch/release"
ref=$(sed -n 's/^ref=//p' "$scratch/release/release-provenance.txt")
expected=$(sed -n 's/^commit=//p' "$scratch/release/release-provenance.txt")
client() { env XDG_DATA_HOME="$scratch/$1/data" XDG_CONFIG_HOME="$scratch/$1/config" flatpak "${@:2}"; }
client trusted remote-add --user --gpg-import="$scratch/release/release-key.gpg" studio-verification "file://$scratch/release/repository"
test "$(client trusted remote-info --user --show-commit studio-verification "$ref")" = "$expected"
client trusted install --user --noninteractive --no-deps studio-verification "$ref"
test "$(client trusted info --user --show-commit org.aloworld.Studio)" = "$expected"
# A different key must not authenticate the same signed repository.
gpg --batch --pinentry-mode loopback --passphrase '' --quick-generate-key 'Untrusted repository verification only' ed25519 sign 1d
other=$(gpg --batch --with-colons --list-secret-keys 'Untrusted repository verification only' | awk -F: '$1 == "fpr" { print $10; exit }')
test "$other" != "$fingerprint"
gpg --batch --export "$other" > "$scratch/wrong-key.gpg"
if { client untrusted remote-add --user --gpg-import="$scratch/wrong-key.gpg" studio-verification "file://$scratch/release/repository" && client untrusted remote-info --user --show-commit studio-verification "$ref"; } > "$scratch/untrusted.log" 2>&1; then
  printf 'A repository authenticated with the wrong signing key.\n' >&2; exit 1
fi
grep -Eq 'GPG|[Ss]ignature|[Ss]igned' "$scratch/untrusted.log" || { cat "$scratch/untrusted.log" >&2; exit 1; }
# Damaged metadata must also be refused with the correct key.
printf 'tampered summary\n' > "$scratch/release/repository/summary"
if [[ -e "$scratch/release/repository/summary.idx" ]]; then
  printf 'tampered summary index\n' > "$scratch/release/repository/summary.idx"
fi
if { client tampered remote-add --user --gpg-import="$scratch/release/release-key.gpg" studio-verification "file://$scratch/release/repository" && client tampered remote-info --user --show-commit studio-verification "$ref"; } > "$scratch/tampered.log" 2>&1; then
  printf 'Tampered repository metadata was accepted.\n' >&2; exit 1
fi
grep -Eq 'summary|metadata|GVariant|GPG|[Ss]ignature|[Ss]igned' "$scratch/tampered.log" || { cat "$scratch/tampered.log" >&2; exit 1; }
printf 'SIGNED_REPOSITORY_INSTALL_VERIFIED\nWRONG_SIGNING_KEY_REFUSED\nTAMPERED_REPOSITORY_METADATA_REFUSED\nTEST_KEYS_AND_CLIENTS_REMOVED_ON_EXIT\n'
