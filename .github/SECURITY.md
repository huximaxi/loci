# Security Policy

The Loci project takes security seriously. This file is the GitHub-side mirror of the operational policy that lives at [loci.garden/.well-known/security.txt](https://loci.garden/.well-known/security.txt) (RFC 9116).

## Reporting a vulnerability

If you have found a vulnerability in the Loci desktop application, the loci.garden site, the MCP server, or any other component of this project, please **do not** open a public issue.

Choose one of the following channels:

1. **Preferred: GitHub Security Advisories**
   Open a private advisory at [github.com/huximaxi/Loci/security/advisories/new](https://github.com/huximaxi/Loci/security/advisories/new). This keeps the report private to the maintainers until a fix is available and a coordinated disclosure date is agreed.

2. **Email**
   [hux@nymtech.net](mailto:hux@nymtech.net) (primary) or [daniel.nemet@nymtech.net](mailto:daniel.nemet@nymtech.net) (backup). Encrypted email is welcome; the project's PGP key publication is in progress.

3. **Machine-readable contact**
   [loci.garden/.well-known/security.txt](https://loci.garden/.well-known/security.txt). Current expiry is 2027-05-16; renewed annually.

## What is in scope

- The Loci desktop application (Tauri v2, Rust + WebView frontend) and its local MCP server.
- The loci.garden static site and any subdomains under `loci.garden`.
- The CONTRIBUTING.md charter (the Rainbow Zoku charter) and any review/merge workflow it documents.
- Build and release artefacts published via GitHub Releases on this repository.

## What is out of scope

- The palace-vps host operating system and infrastructure operated by 1984 Hosting (Iceland). Report directly to [1984 Hosting](https://1984.hosting/) for host-level issues.
- Third-party services the project links to (Nym, Anthropic, Ollama, Goose, Block, etc).
- Issues in the methodology or in dispatches as written work, unless they enable a concrete privacy or security harm to a reader.
- Anything you found while violating the Rainbow Zoku charter (sock-puppet review, misrepresented authorship, etc) is not a vulnerability to report; it is a charter violation to disclose.

## What to expect

- **Acknowledgement** within 72 hours.
- **Triage** within one week.
- **Coordinated disclosure** timeline negotiated case-by-case. The default is 90 days from acknowledgement to public disclosure, accelerated if a fix is shipped sooner, extended if active exploitation is suspected.
- **Credit** in the advisory and release notes if you want it. Pseudonymous credit is fully accepted; the Rainbow Zoku charter is pseudonym-friendly by design.

## Bounty

There is no monetary bounty programme. The Loci project is a commons; the work belongs to whoever tends it. We do offer:
- Credit in release notes
- A first-call invitation to review related code paths
- The thanks of every future user whose machine your report protected

## Coordinated disclosure principles

We commit to:
- Not pursuing legal action against good-faith security researchers
- Not requiring NDAs to receive a report
- Naming the issue clearly in release notes once disclosed
- Acknowledging when we got it wrong if our triage misclassified a report

We expect researchers to:
- Avoid privacy violations, data destruction, or service disruption while testing
- Give us reasonable time to fix before public disclosure
- Not exploit the vulnerability beyond what is needed to demonstrate it
- Not share the vulnerability with third parties until coordinated disclosure

---

Last updated: 2026-05-17 · Authored by the Rainbow Zoku in commons.
