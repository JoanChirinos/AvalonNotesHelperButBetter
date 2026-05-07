# Avalon Notes Helper

Real-time note-taking app for **Avalon: The Resistance (Big Box Edition)**. Designed to be projected on a screen during physical gameplay. Multiple clients connect via WebSocket and see live updates — one person takes notes, everyone sees them instantly.

## Features

- Full game lifecycle: setup, active gameplay, assassination, role reveal
- All 22 roles (Big Box Edition) with bundle enforcement (Messengers, Sorcerers, Lancelots)
- All 3 modules: Lady of the Lake, Lancelot Switching, Plot Cards
- Real-time sync across all connected clients via WebSocket
- Quest results auto-derived from card counts + magic + fail thresholds
- Every round recorded: king, team, votes, card counts
- Everything is editable (it's a note-taking app — mistakes happen)
- Round timer with alarm
- Tools: random king, random team, window-based team selection
- Dark/light theme
- Works on phones, tablets, and projected displays

## Quick Start (Development)

**Prerequisites:** Rust toolchain, Node.js 24+

```bash
# Terminal 1: backend (port 8337)
cd backend && cargo run

# Terminal 2: frontend dev server (port 3817, proxies /api to backend)
cd frontend && npm install && npm run dev
```

Open `http://localhost:3817`.

## Deployment

### Build

```bash
# Backend
cd backend && cargo build --release

# Frontend
cd frontend && npm install && VITE_BASE_PATH=/ npm run build
```

The backend produces a single binary at `backend/target/release/avalon-notes`. The frontend produces static files in `frontend/dist/`.

### Configuration

**`VITE_BASE_PATH`** (build-time env var) controls what URL prefix the app lives under:
- `/` — served at the domain root (e.g. `https://avalon.example.com/`)
- `/anh3/` — served under a subpath (e.g. `https://example.com/anh3/`)

Must start and end with `/`.

### Running the Backend

Run the binary from the `backend/` directory (it creates/reads `avalon.db` in the working directory):

```bash
cd backend && ./target/release/avalon-notes
```

Listens on `0.0.0.0:8337`. Use your OS's process manager (systemd, launchd, etc.) to keep it alive.

### Reverse Proxy Setup

You need a reverse proxy that does two things:

1. **Proxy API + WebSocket** to the backend (port 8337), stripping the subpath if applicable
2. **Serve static files** from `frontend/dist/` with SPA fallback to `index.html`

#### nginx (subpath example: `/anh3/`)

```nginx
# API + WebSocket
location /anh3/api/ {
    proxy_pass http://127.0.0.1:8337/api/;
    proxy_http_version 1.1;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection 'upgrade';
    proxy_cache_bypass $http_upgrade;
}

# Static files (SPA fallback)
location /anh3/ {
    root /path/to/www;  # must contain "anh3/" dir with dist contents
    try_files $uri $uri/ /anh3/index.html;
}
```

Note: nginx `root` expects the subpath as a subdirectory name. Symlink or copy `frontend/dist/` to `/path/to/www/anh3/`.

#### nginx (root domain example)

```nginx
location /api/ {
    proxy_pass http://127.0.0.1:8337/api/;
    proxy_http_version 1.1;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection 'upgrade';
    proxy_cache_bypass $http_upgrade;
}

location / {
    root /path/to/frontend/dist;
    try_files $uri $uri/ /index.html;
}
```

#### Caddy (root domain)

```
example.com {
    handle /api/* {
        reverse_proxy localhost:8337
    }
    handle {
        root * /path/to/frontend/dist
        try_files {path} /index.html
        file_server
    }
}
```

### Permissions

If your reverse proxy runs as a different user, ensure the dist directory and its parent directories are readable:

```bash
chmod -R o+r /path/to/frontend/dist
# All parent dirs need execute for traversal
chmod o+x /path/to/project /path/to/project/frontend /path/to/project/frontend/dist
```

## Tech Stack

- **Backend:** Rust, Axum 0.8, SQLite (Diesel ORM), WebSocket broadcast via tokio
- **Frontend:** Svelte 5 (runes), Vite 8, TypeScript, Tailwind CSS 4, DaisyUI 5
- **Real-time:** Full game state broadcast on every mutation (~10-20KB)

## Game Rules

Supports the complete Big Box Edition ruleset:

- 5-10 players, up to 5 quests, up to 5 proposals per quest
- Quest 4 with 7+ players requires 2 fail cards
- Magic cards flip quest result (odd = flip, even = no change)
- End-of-game message resolution (3+ good messages or 2+ evil messages modify Quest 5)
- Two-phase assassination (Untrustworthy Servant identification, then Merlin/Messengers snipe)
- House rule: 5 consecutive rejections fails the quest (not the game)
