# loci/server

Lightweight FastAPI waitlist service for Loci early access signups.

Ported from `Dev/wedding-server/` — same stack, stripped to essentials.

## Stack

- FastAPI + uvicorn
- SQLite (single `waitlist.db` file — no migrations needed)
- SMTP notification to Hux on each signup (optional, configured via env)
- Privacy-first: email only, IP stored as SHA256 hash

## Run locally

```bash
cd Dev/loci/server
python -m venv venv && source venv/bin/activate
pip install -r requirements.txt
cp .env.example .env  # fill in LOCI_ADMIN_TOKEN
uvicorn main:app --host 127.0.0.1 --port 8001 --reload
```

## Endpoints

| Method | Path | Auth | What |
|--------|------|------|------|
| POST | `/waitlist` | none | Submit email signup |
| GET | `/waitlist/count` | none | Public count (optional social proof) |
| GET | `/admin/waitlist?token=...` | query token | List all entries |
| GET | `/health` | none | Health + entry count |

## Deploy (VPS)

Same pattern as wedding-server:

```
# systemd unit: loci-waitlist.service
[Unit]
Description=Loci Waitlist Service
After=network.target

[Service]
User=hux
WorkingDirectory=/home/hux/loci-server
EnvironmentFile=/home/hux/loci-server/.env
ExecStart=/home/hux/loci-server/venv/bin/uvicorn main:app --host 127.0.0.1 --port 8001
Restart=always

[Install]
WantedBy=multi-user.target
```

Caddy reverse proxy:
```
loci.garden {
    # ... existing landing config ...
    handle /waitlist {
        reverse_proxy 127.0.0.1:8001
    }
    handle /waitlist/count {
        reverse_proxy 127.0.0.1:8001
    }
}
```

## loci.garden form integration

```html
<form id="waitlist-form">
  <input type="email" name="email" placeholder="your@email.com" required />
  <button type="submit">Get early access</button>
</form>
<script>
document.getElementById('waitlist-form').addEventListener('submit', async (e) => {
  e.preventDefault();
  const email = e.target.email.value;
  const res = await fetch('https://loci.garden/waitlist', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, source: 'loci.garden/hero' })
  });
  const data = await res.json();
  // show data.message to user
});
</script>
```
