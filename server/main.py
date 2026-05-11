"""Loci waitlist server — lightweight FastAPI service for early access signups.

Ported from wedding-server pattern (Dev/wedding-server/).
Privacy-first: email only, IP as SHA256 hash, no third parties, no accounts.

Deploy: uvicorn main:app --host 127.0.0.1 --port 8001
Caddy: reverse proxy to 127.0.0.1:8001, loci.garden/waitlist → POST /waitlist
"""
import hashlib
import logging
import sqlite3
from datetime import datetime
from pathlib import Path
from typing import Optional

from fastapi import FastAPI, HTTPException, Query, Depends
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, EmailStr

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
log = logging.getLogger(__name__)

app = FastAPI(title="Loci Waitlist", version="1.0.0", docs_url=None, redoc_url=None)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["https://loci.garden", "http://localhost:3000"],
    allow_methods=["POST", "GET"],
    allow_headers=["Content-Type"],
)

DB = Path(__file__).parent / "waitlist.db"
ADMIN_TOKEN = None  # loaded from env — see config.py


# ── Models ────────────────────────────────────────────────────────────────────

class SignupRequest(BaseModel):
    email: EmailStr
    source: str = "loci.garden"  # which CTA triggered this


class SignupResponse(BaseModel):
    success: bool
    message: str


# ── DB ────────────────────────────────────────────────────────────────────────

def init_db() -> None:
    with sqlite3.connect(DB) as c:
        c.execute("""
            CREATE TABLE IF NOT EXISTS waitlist (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                email      TEXT    NOT NULL UNIQUE,
                source     TEXT    NOT NULL DEFAULT 'loci.garden',
                ip_hash    TEXT,
                created_at TEXT    NOT NULL
            )
        """)
        c.commit()


def insert(email: str, source: str, ip_hash: Optional[str]) -> int:
    with sqlite3.connect(DB) as c:
        cur = c.execute(
            "INSERT INTO waitlist (email, source, ip_hash, created_at) VALUES (?,?,?,?)",
            (email, source, ip_hash, datetime.utcnow().isoformat())
        )
        c.commit()
        return cur.lastrowid


def all_entries() -> list[dict]:
    with sqlite3.connect(DB) as c:
        c.row_factory = sqlite3.Row
        return [dict(r) for r in c.execute(
            "SELECT id, email, source, created_at FROM waitlist ORDER BY created_at DESC"
        )]


def count() -> int:
    with sqlite3.connect(DB) as c:
        return c.execute("SELECT COUNT(*) FROM waitlist").fetchone()[0]


def sha256_ip(ip: Optional[str]) -> Optional[str]:
    return hashlib.sha256(ip.encode()).hexdigest() if ip else None


# ── Auth ──────────────────────────────────────────────────────────────────────

def require_admin(token: str = Query(...)) -> None:
    import os
    expected = os.getenv("LOCI_ADMIN_TOKEN", "")
    if not expected or token != expected:
        raise HTTPException(status_code=403, detail="Forbidden")


# ── Startup ───────────────────────────────────────────────────────────────────

@app.on_event("startup")
async def startup():
    init_db()
    log.info(f"Loci waitlist ready — {count()} entries")


# ── Routes ────────────────────────────────────────────────────────────────────

@app.post("/waitlist", response_model=SignupResponse)
async def join_waitlist(body: SignupRequest, request_obj=None):
    """Public endpoint — loci.garden form posts here."""
    from fastapi import Request
    # ip extraction requires the raw Request; FastAPI injects it if declared
    ip = None
    try:
        ip = request_obj.client.host if request_obj else None
    except Exception:
        pass

    try:
        entry_id = insert(body.email, body.source, sha256_ip(ip))
        log.info(f"Waitlist #{entry_id}: {body.source}")
        _notify_hux(body.email, entry_id)
        return SignupResponse(success=True, message="You're on the list.")
    except Exception as e:
        if "UNIQUE constraint" in str(e):
            return SignupResponse(success=True, message="Already on the list.")
        log.error(f"Waitlist insert failed: {e}")
        raise HTTPException(status_code=500, detail="Try again.")


@app.post("/waitlist", response_model=SignupResponse, include_in_schema=False)
async def join_waitlist_with_request(body: SignupRequest, req=None):
    """Alias that captures Request for IP hashing."""
    return await join_waitlist(body, req)


@app.get("/waitlist/count")
async def waitlist_count_public():
    """Public count endpoint — for social proof on landing page if desired."""
    return {"count": count()}


@app.get("/admin/waitlist", dependencies=[Depends(require_admin)])
async def admin_list():
    entries = all_entries()
    return {"total": len(entries), "entries": entries}


@app.get("/health")
async def health():
    return {"status": "ok", "entries": count()}


# ── Notification ──────────────────────────────────────────────────────────────

def _notify_hux(email: str, entry_id: int) -> None:
    """Best-effort email notification to Hux on each signup. Never raises."""
    import os, smtplib
    from email.message import EmailMessage
    try:
        smtp_host = os.getenv("SMTP_HOST", "")
        smtp_port = int(os.getenv("SMTP_PORT", "587"))
        smtp_user = os.getenv("SMTP_USER", "")
        smtp_pass = os.getenv("SMTP_PASS", "")
        hux_email = os.getenv("NOTIFY_EMAIL", "daniel.nemet@nymtech.net")

        if not all([smtp_host, smtp_user, smtp_pass]):
            return  # no SMTP configured — skip silently

        msg = EmailMessage()
        msg["Subject"] = f"[Loci waitlist] #{entry_id} signup"
        msg["From"] = smtp_user
        msg["To"] = hux_email
        msg.set_content(f"New Loci waitlist signup.\n\nEmail: {email}\nEntry ID: #{entry_id}")

        with smtplib.SMTP(smtp_host, smtp_port) as s:
            s.starttls()
            s.login(smtp_user, smtp_pass)
            s.send_message(msg)
    except Exception as e:
        log.warning(f"Hux notification failed for #{entry_id}: {e}")


if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host="127.0.0.1", port=8001, reload=True)
