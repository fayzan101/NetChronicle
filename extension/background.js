const AGENT_URL = "http://127.0.0.1:9477/browser-tab";
const REPORT_INTERVAL_MS = 2000;

const SKIP_PREFIXES = ["chrome://", "edge://", "about:", "chrome-extension://"];

async function reportActiveTab() {
  try {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab?.url) return;
    if (SKIP_PREFIXES.some((prefix) => tab.url.startsWith(prefix))) return;

    await fetch(AGENT_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        url: tab.url,
        title: tab.title ?? null,
        tabId: tab.id ?? null,
        active: true,
      }),
    });
  } catch {
    // Agent offline — extension keeps running silently.
  }
}

chrome.tabs.onActivated.addListener(() => {
  reportActiveTab();
});

chrome.tabs.onUpdated.addListener((_tabId, changeInfo, tab) => {
  if (changeInfo.status === "complete" && tab.active) {
    reportActiveTab();
  }
});

chrome.windows.onFocusChanged.addListener((windowId) => {
  if (windowId !== chrome.windows.WINDOW_ID_NONE) {
    reportActiveTab();
  }
});

setInterval(reportActiveTab, REPORT_INTERVAL_MS);
