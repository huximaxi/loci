#!/usr/bin/env bash
# release-sums.sh: write, sign and verify a SHA256SUMS file for release artifacts.
#
#   release-sums.sh sign   <dir> <private-key>          writes <dir>/SHA256SUMS and SHA256SUMS.sig
#   release-sums.sh verify <dir> <allowed_signers> [principal]
#   release-sums.sh selftest                            throwaway key, proves each failure mode
#
# Signing uses `ssh-keygen -Y sign` in the namespace `loci-release`. Verification
# needs only the public key, listed in an allowed_signers file. Offline, no network.
# The private key is read by ssh-keygen and never copied or printed here.
#
# What counts as an artifact: every regular, non-hidden file directly inside <dir>.
# Hidden files (.DS_Store and friends) are never artifacts and are skipped by both
# sides. Subfolders, and file names holding a backslash or a newline, are refused
# rather than dropped, so nothing ships outside the sums by accident.
# `verify` is strict both ways: every listed file must match, and every artifact in
# the folder must be listed.

set -euo pipefail

NAMESPACE="loci-release"
DEFAULT_PRINCIPAL="loci-maintainer"

die() { printf 'release-sums: %s\n' "$*" >&2; exit 1; }

sha256_line() { # one "hash  name" line for a file in the current directory
  if command -v sha256sum >/dev/null 2>&1; then sha256sum -- "$1"
  else shasum -a 256 -- "$1"; fi
}

sums_check() { # run inside the artifact folder
  if command -v sha256sum >/dev/null 2>&1; then sha256sum --strict -c SHA256SUMS
  else shasum -a 256 --strict -c SHA256SUMS; fi
}

# Print the artifact names in <dir>, one per line, sorted. Dies on anything unsafe.
list_artifacts() {
  local dir=$1 f name
  [ -d "$dir" ] || die "not a directory: $dir"
  while IFS= read -r -d '' f; do
    name=${f#"$dir"/}
    case "$name" in
      .*) continue ;;                                   # hidden: not an artifact
      SHA256SUMS|SHA256SUMS.sig) continue ;;
    esac
    case "$name" in
      *'\'*|*$'\n'*) die "refusing a file name with a backslash or newline: $name" ;;
    esac
    if [ -d "$f" ]; then die "refusing subfolder (artifacts sit directly in the folder): $name"; fi
    [ -f "$f" ] || die "not a regular file: $name"
    printf '%s\n' "$name"
  done < <(find "$dir" -mindepth 1 -maxdepth 1 -print0) | LC_ALL=C sort
}

write_sums() {
  local dir=$1 names tmp name
  # Explicit exits: a die inside the listing runs in a subshell, and `set -e` is
  # switched off whenever a caller sits in an `if`, so do not rely on it.
  names=$(list_artifacts "$dir") || exit 1               # aborts before any write
  [ -n "$names" ] || die "no artifacts in $dir"
  tmp=$(mktemp "$dir/.SHA256SUMS.XXXXXX") || exit 1
  ( cd "$dir"
    while IFS= read -r name; do sha256_line "$name"; done <<EOF
$names
EOF
  ) > "$tmp" || { rm -f "$tmp"; die "hashing failed, nothing written"; }
  mv "$tmp" "$dir/SHA256SUMS" || exit 1
}

# Names listed in SHA256SUMS ("hash  name" lines), sorted.
listed_names() {
  sed -E 's/^[0-9a-fA-F]{64}  //' "$1/SHA256SUMS" | LC_ALL=C sort
}

cmd_sign() {
  local dir=${1:-} key=${2:-}
  [ -n "$dir" ] && [ -n "$key" ] || die "usage: sign <dir> <private-key>"
  [ -f "$key" ] || die "no such key file: $key"
  write_sums "$dir" || exit 1
  rm -f "$dir/SHA256SUMS.sig"
  ssh-keygen -Y sign -f "$key" -n "$NAMESPACE" "$dir/SHA256SUMS" >/dev/null
  printf 'signed: %s/SHA256SUMS (%s artifacts)\n' "$dir" "$(wc -l < "$dir/SHA256SUMS" | tr -d ' ')"
  printf 'next: run `verify` against the published allowed_signers; a key that is not listed there only fails for your users.\n'
}

cmd_verify() {
  local dir=${1:-} signers=${2:-} principal=${3:-$DEFAULT_PRINCIPAL} listed found n
  [ -n "$dir" ] && [ -n "$signers" ] || die "usage: verify <dir> <allowed_signers> [principal]"
  [ -f "$dir/SHA256SUMS" ] || die "missing $dir/SHA256SUMS"
  [ -f "$dir/SHA256SUMS.sig" ] || die "missing $dir/SHA256SUMS.sig"
  [ -f "$signers" ] || die "no such allowed_signers file: $signers"
  # 1. the signature covers the sums file as published
  ssh-keygen -Y verify -f "$signers" -I "$principal" -n "$NAMESPACE" \
    -s "$dir/SHA256SUMS.sig" < "$dir/SHA256SUMS" >/dev/null 2>&1 \
    || die "signature does not verify for principal $principal"
  # 2. the sums name exactly the artifacts in the folder, and at least one
  listed=$(listed_names "$dir")
  [ -n "$listed" ] || die "SHA256SUMS lists no artifacts"
  found=$(list_artifacts "$dir") || exit 1
  [ "$listed" = "$found" ] || die "folder and SHA256SUMS disagree: $(
    diff <(printf '%s\n' "$listed") <(printf '%s\n' "$found") | grep '^[<>]' | tr '\n' ';')"
  # 3. every listed artifact matches its hash
  ( cd "$dir" && sums_check >/dev/null 2>&1 ) || die "an artifact does not match SHA256SUMS"
  n=$(printf '%s\n' "$listed" | wc -l | tr -d ' ')
  printf 'verified: signature ok, %s artifacts present and matching\n' "$n"
}

cmd_selftest() {
  local pass=0 fail=0 got err
  t=$(mktemp -d)
  trap 'rm -rf "$t"' EXIT
  mkdir "$t/rel"
  ssh-keygen -q -t ed25519 -N '' -C selftest -f "$t/key"
  ssh-keygen -q -t ed25519 -N '' -C other -f "$t/otherkey"
  printf '%s %s\n' "$DEFAULT_PRINCIPAL" "$(cut -d' ' -f1,2 "$t/key.pub")" > "$t/allowed"
  printf '%s %s\n' "$DEFAULT_PRINCIPAL" "$(cut -d' ' -f1,2 "$t/otherkey.pub")" > "$t/allowed-other"
  printf 'binary one\n' > "$t/rel/loci-linux"
  printf 'binary two\n' > "$t/rel/loci-macos"
  printf 'finder litter\n' > "$t/rel/.DS_Store"          # hidden: must not matter
  printf 'three\n' > "$t/rel/loci with space.zip"        # a space in a name must work
  cmd_sign "$t/rel" "$t/key" >/dev/null 2>&1

  # expect <ok|fail> <label> <stderr-fragment-or-""> <cmd...>
  expect() {
    local want=$1 label=$2 frag=$3; shift 3
    if err=$("$@" 2>&1 >/dev/null); then got=ok; else got=fail; fi
    if [ "$got" = "$want" ] && { [ -z "$frag" ] || printf '%s' "$err" | grep -qF -- "$frag"; }; then
      pass=$((pass+1)); printf '  pass: %s\n' "$label"
    else
      fail=$((fail+1)); printf '  FAIL: %s (wanted %s %s, got %s: %s)\n' "$label" "$want" "$frag" "$got" "$err"
    fi
  }
  v() ( cmd_verify "$@" )    # subshell so die() cannot end the selftest
  s() ( cmd_sign "$@" )

  expect ok   "untouched release verifies (hidden file ignored, spaced name ok)" "" v "$t/rel" "$t/allowed"
  expect fail "wrong signer key is refused"          "signature does not verify" v "$t/rel" "$t/allowed-other"
  expect fail "wrong principal is refused"           "signature does not verify" v "$t/rel" "$t/allowed" someone-else

  cp -R "$t/rel" "$t/tamper-file"
  printf 'x' >> "$t/tamper-file/loci-linux"
  expect fail "a modified artifact is refused"       "does not match" v "$t/tamper-file" "$t/allowed"

  # Replace a file AND its sums line consistently, keep the old signature:
  # only the signature check can catch this.
  cp -R "$t/rel" "$t/swap"
  printf 'evil\n' > "$t/swap/loci-linux"
  ( cd "$t/swap" && { sha256_line loci-linux; sha256_line loci-macos; sha256_line "loci with space.zip"; } > SHA256SUMS )
  expect fail "a consistently rewritten sums file is refused (signature only)" "signature does not verify" v "$t/swap" "$t/allowed"

  cp -R "$t/rel" "$t/no-sig"
  rm "$t/no-sig/SHA256SUMS.sig"
  expect fail "a missing signature is refused"       "missing" v "$t/no-sig" "$t/allowed"

  cp -R "$t/rel" "$t/extra"
  printf 'evil\n' > "$t/extra/evil.exe"
  expect fail "an unlisted extra file is refused"    "disagree" v "$t/extra" "$t/allowed"

  cp -R "$t/rel" "$t/gone"
  rm "$t/gone/loci-macos"
  expect fail "a missing listed file is refused"     "disagree" v "$t/gone" "$t/allowed"

  mkdir "$t/sub" && cp "$t/rel/loci-linux" "$t/sub/" && mkdir "$t/sub/nested"
  expect fail "sign refuses a subfolder"             "subfolder" s "$t/sub" "$t/key"
  [ ! -e "$t/sub/SHA256SUMS" ] && { pass=$((pass+1)); echo "  pass: a refused sign leaves no SHA256SUMS behind"; } \
    || { fail=$((fail+1)); echo "  FAIL: a refused sign left a SHA256SUMS behind"; }

  mkdir "$t/empty"
  expect fail "sign refuses an empty folder"         "no artifacts" s "$t/empty" "$t/key"

  printf '\nselftest: %s passed, %s failed\n' "$pass" "$fail"
  [ "$fail" -eq 0 ]
}

case "${1:-}" in
  sign)     shift; cmd_sign "$@" ;;
  verify)   shift; cmd_verify "$@" ;;
  selftest) cmd_selftest ;;
  *) die "usage: release-sums.sh sign|verify|selftest (see header)" ;;
esac
