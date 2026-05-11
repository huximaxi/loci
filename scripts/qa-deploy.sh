#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
# loci.garden deployment QA
# Usage:
#   ./scripts/qa-deploy.sh                  # standard checks
#   ./scripts/qa-deploy.sh --post-test      # includes canary signup POST
#   DOMAIN=staging.loci.garden ./scripts/qa-deploy.sh
# ─────────────────────────────────────────────────────────────

DOMAIN="${DOMAIN:-loci.garden}"
BASE="https://$DOMAIN"
PASS=0
FAIL=0
TIMEOUT=10

# ── Colours ──────────────────────────────────────────────────
GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

ok()   { echo -e "  ${GREEN}✓${NC}  $1"; PASS=$((PASS+1)); }
fail() { echo -e "  ${RED}✗${NC}  $1"; FAIL=$((FAIL+1)); }
info() { echo -e "\n${CYAN}${BOLD}$1${NC}"; }

# ── Helpers ───────────────────────────────────────────────────
check_status() {
  local label="$1" url="$2" expected="${3:-200}"
  local status
  status=$(curl -s -o /dev/null -w "%{http_code}" --max-time "$TIMEOUT" "$url" 2>/dev/null)
  if [[ "$status" == "$expected" ]]; then
    ok "$label  ${YELLOW}[$status]${NC}"
  else
    fail "$label  expected=$expected got=${status:-timeout}  → $url"
  fi
}

check_body() {
  local label="$1" url="$2" pattern="$3"
  local body
  body=$(curl -s --max-time "$TIMEOUT" "$url" 2>/dev/null)
  if echo "$body" | grep -q "$pattern"; then
    ok "$label"
  else
    fail "$label  pattern '${pattern}' not found  → $url"
  fi
}

check_redirect() {
  local label="$1" url="$2" expected_dest="$3" expected_code="${4:-301}"
  local status dest
  status=$(curl -s -o /dev/null -w "%{http_code}" --max-time "$TIMEOUT" "$url" 2>/dev/null)
  dest=$(curl -s -o /dev/null -w "%{redirect_url}" --max-time "$TIMEOUT" "$url" 2>/dev/null)
  if [[ "$status" == "$expected_code" ]] && echo "$dest" | grep -q "$expected_dest"; then
    ok "$label  ${YELLOW}[$status → $dest]${NC}"
  else
    fail "$label  expected=$expected_code→$expected_dest got=${status}→${dest:-none}  → $url"
  fi
}

# ── Header ────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}╔═══════════════════════════════════════╗${NC}"
echo -e "${BOLD}║   loci.garden deployment QA           ║${NC}"
echo -e "${BOLD}╚═══════════════════════════════════════╝${NC}"
echo -e "  Domain : ${BOLD}$BASE${NC}"
echo -e "  Run at : $(date -u '+%Y-%m-%d %H:%M UTC')"

# ── Landing ───────────────────────────────────────────────────
info "Landing page"
check_status  "GET /"                   "$BASE/"               200
check_body    "/ contains 'loci'"       "$BASE/"               "loci"
check_status  "GET /manifesto.html"     "$BASE/manifesto.html" 200
check_status  "GET /roadmap.html"       "$BASE/roadmap.html"   200

# ── Redirects ─────────────────────────────────────────────────
info "Redirects"
check_redirect "www → apex" "https://www.$DOMAIN/" "$DOMAIN" 301

# ── Waitlist API ──────────────────────────────────────────────
info "Waitlist API"
check_status "GET /waitlist/count"              "$BASE/waitlist/count" 200
check_body   "/waitlist/count returns {count}"  "$BASE/waitlist/count" '"count"'

if [[ "${1:-}" == "--post-test" ]]; then
  echo -e "  ${YELLOW}→${NC} canary signup (--post-test)"
  response=$(curl -s -X POST "$BASE/waitlist" \
    -H "Content-Type: application/json" \
    -d '{"email":"qa-canary@loci.garden","source":"qa-script"}' \
    --max-time "$TIMEOUT" 2>/dev/null)
  if echo "$response" | grep -q '"success":true'; then
    ok "POST /waitlist — canary accepted"
  else
    fail "POST /waitlist — unexpected: $response"
  fi
fi

# ── Docs ──────────────────────────────────────────────────────
info "Docs"
check_status "GET docs.$DOMAIN" "https://docs.$DOMAIN/" 200

# ── Summary ───────────────────────────────────────────────────
echo ""
echo -e "───────────────────────────────────────"
TOTAL=$((PASS+FAIL))
if [[ $FAIL -eq 0 ]]; then
  echo -e "  ${GREEN}${BOLD}ALL $PASS/$TOTAL checks passed${NC}  ✦"
  echo ""
  exit 0
else
  echo -e "  ${RED}${BOLD}$FAIL/$TOTAL failed${NC}  ($PASS passed)"
  echo ""
  exit 1
fi
