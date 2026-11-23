#!/usr/bin/env bash
# release-sums.sh: write, sign and verify a SHA256SUMS file for release artifacts.
#
#   release-sums.sh sign   <dir> <private-key>          writes <dir>/SHA256SUMS and SHA256SUMS.sig
#   release-sums.sh verify <dir> <allowed_signers> [principal]
#   release-sums.sh selftest                            throwaway key, proves a tampered file fails
#
# Signing uses `ssh-keygen -Y sign` in the namespace `loci-release`. Verification
# needs only the public key, listed in an allowed_signers file. Offline, no network.
# The private key is read by ssh-keygen and never copied or printed here.

set -euo pipefail

NAMESPACE="loci-release"
DEFAULT_PRINCIPAL="loci-maintainer"

die() { printf 'release-sums: %s\n' "$*" >&2; exit 1; }

sha256_cmd() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$@"
  elif command -v shasum >/dev/null 2>&1; then shasum -a 256 "$@"
  else die "need sha256sum or shasum"; fi
}

sums_check() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum -c SHA256SUMS
  else shasum -a 256 -c SHA256SUMS; fi
}

write_sums() {
  local dir=$1
  [ -d "$dir" ] || die "not a directory: $dir"
  ( cd "$dir"
    # Everything in the folder except the sums and the signature, sorted, names only.
    local files
    files=$(find . -maxdepth 1 -type f ! -name SHA256SUMS ! -name SHA256SUMS.sig | sed 's|^\./||' | LC_ALL=C sort)
    [ -n "$files" ] || die "no artifacts in $dir"
    # shellcheck disable=SC2086
    printf '%s\n' $files | while IFS= read -r f; do sha256_cmd "$f"; done > SHA256SUMS
  )
}

cmd_sign() {
  local dir=${1:-} key=${2:-}
  [ -n "$dir" ] && [ -n "$key" ] || die "usage: sign <dir> <private-key>"
  [ -f "$key" ] || die "no such key file: $key"
  write_sums "$dir"
  rm -f "$dir/SHA256SUMS.sig"
  ssh-keygen -Y sign -f "$key" -n "$NAMESPACE" "$dir/SHA256SUMS" >/dev/null
  printf 'signed: %s/SHA256SUMS (%s lines)\n' "$dir" "$(wc -l < "$dir/SHA256SUMS" | tr -d ' ')"
}

cmd_verify() {
  local dir=${1:-} signers=${2:-} principal=${3:-$DEFAULT_PRINCIPAL}
  [ -n "$dir" ] && [ -n "$signers" ] || die "usage: verify <dir> <allowed_signers> [principal]"
  [ -f "$dir/SHA256SUMS" ] || die "missing $dir/SHA256SUMS"
  [ -f "$dir/SHA256SUMS.sig" ] || die "missing $dir/SHA256SUMS.sig"
  [ -f "$signers" ] || die "no such allowed_signers file: $signers"
  # 1. the signature covers the sums file as published
  ssh-keygen -Y verify -f "$signers" -I "$principal" -n "$NAMESPACE" \
    -s "$dir/SHA256SUMS.sig" < "$dir/SHA256SUMS" >/dev/null \
    || die "signature does not verify for principal $principal"
  # 2. the sums cover the artifacts as downloaded
  ( cd "$dir" && sums_check >/dev/null ) || die "an artifact does not match SHA256SUMS"
  printf 'verified: signature ok, %s artifacts match\n' "$(wc -l < "$dir/SHA256SUMS" | tr -d ' ')"
}

cmd_selftest() {
  local pass=0 fail=0
  t=$(mktemp -d)
  trap 'rm -rf "$t"' EXIT
  mkdir "$t/rel" "$t/other"
  ssh-keygen -q -t ed25519 -N '' -C selftest -f "$t/key"
  ssh-keygen -q -t ed25519 -N '' -C other -f "$t/otherkey"
  printf '%s %s\n' "$DEFAULT_PRINCIPAL" "$(cut -d' ' -f1,2 "$t/key.pub")" > "$t/allowed"
  printf '%s %s\n' "$DEFAULT_PRINCIPAL" "$(cut -d' ' -f1,2 "$t/otherkey.pub")" > "$t/allowed-other"
  printf 'binary one\n' > "$t/rel/loci-linux"
  printf 'binary two\n' > "$t/rel/loci-macos"
  cmd_sign "$t/rel" "$t/key" >/dev/null 2>&1

  expect() { # expect <ok|fail> <label> <cmd...>
    local want=$1 label=$2; shift 2
    if "$@" >/dev/null 2>&1; then got=ok; else got=fail; fi
    if [ "$got" = "$want" ]; then pass=$((pass+1)); printf '  pass: %s\n' "$label"
    else fail=$((fail+1)); printf '  FAIL: %s (wanted %s, got %s)\n' "$label" "$want" "$got"; fi
  }
  # Run verify in a subshell so die() cannot end the selftest.
  v() ( cmd_verify "$@" )

  expect ok   "untouched release verifies"             v "$t/rel" "$t/allowed"
  expect fail "wrong signer key is refused"            v "$t/rel" "$t/allowed-other"
  expect fail "wrong principal is refused"             v "$t/rel" "$t/allowed" someone-else

  cp -R "$t/rel" "$t/tamper-file"
  printf 'x' >> "$t/tamper-file/loci-linux"
  expect fail "a modified artifact is refused"         v "$t/tamper-file" "$t/allowed"

  cp -R "$t/rel" "$t/tamper-sums"
  sed -i.bak '1s/^./0/' "$t/tamper-sums/SHA256SUMS" && rm -f "$t/tamper-sums/SHA256SUMS.bak"
  expect fail "an edited SHA256SUMS is refused"        v "$t/tamper-sums" "$t/allowed"

  cp -R "$t/rel" "$t/no-sig"
  rm "$t/no-sig/SHA256SUMS.sig"
  expect fail "a missing signature is refused"         v "$t/no-sig" "$t/allowed"

  printf '\nselftest: %s passed, %s failed\n' "$pass" "$fail"
  [ "$fail" -eq 0 ]
}

case "${1:-}" in
  sign)     shift; cmd_sign "$@" ;;
  verify)   shift; cmd_verify "$@" ;;
  selftest) cmd_selftest ;;
  *) die "usage: release-sums.sh sign|verify|selftest (see header)" ;;
esac
