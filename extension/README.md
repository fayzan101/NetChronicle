# NetChronicle Browser Extension

Chrome / Edge (Manifest V3) extension that sends the active tab URL to the local agent feed.

## Install (unpacked)

1. Start the agent (`cargo run -p netchronicle-agent`) — it listens on `http://127.0.0.1:9477`.
2. Open **Chrome** or **Edge** → `chrome://extensions` / `edge://extensions`.
3. Enable **Developer mode**.
4. Click **Load unpacked** and select this `extension/` folder.

## What it does

Every ~2 seconds (and on tab/window changes), the extension POSTs:

```json
{
  "url": "https://github.com/user/repo",
  "title": "Repository",
  "tabId": 123,
  "active": true
}
```

to `POST http://127.0.0.1:9477/browser-tab`.

The agent prefers this feed over window-title heuristics when the report is fresh (30s).

## Permissions

| Permission | Why |
|------------|-----|
| `tabs` | Read active tab URL/title |
| `activeTab` | Access current tab on change |
| `host_permissions` for `127.0.0.1:9477` | POST to local agent only |

## Configuration

Agent port: `AGENT_BROWSER_FEED_PORT` (default `9477`). Update `AGENT_URL` in `background.js` if you change it.

## Privacy

Data stays on your machine — the extension only talks to the local agent, not the public API or any remote server.
