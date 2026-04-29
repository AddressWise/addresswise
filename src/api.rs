use ntex::http::{StatusCode, header};
use ntex::web;
use ntex::web::HttpResponse;

use crate::error::AppError;
use crate::models::{
    AutocompleteRequest, AutocompleteResponse, ResolveAddressRequest, ResolveAddressResponse,
};
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(resolve_address);
    cfg.service(autocomplete);
    cfg.service(autocomplete_sandbox);
}

#[web::post("/resolve-address")]
async fn resolve_address(
    state: web::types::State<AppState>,
    payload: web::types::Json<ResolveAddressRequest>,
) -> Result<web::types::Json<ResolveAddressResponse>, AppError> {
    let response = state.addresses.resolve(payload.into_inner()).await?;
    Ok(web::types::Json(response))
}

#[web::post("/autocomplete")]
async fn autocomplete(
    state: web::types::State<AppState>,
    payload: web::types::Json<AutocompleteRequest>,
) -> Result<web::types::Json<AutocompleteResponse>, AppError> {
    let response = state.addresses.autocomplete(payload.into_inner()).await?;
    Ok(web::types::Json(response))
}

#[web::get("/sandbox/autocomplete")]
async fn autocomplete_sandbox() -> HttpResponse {
    let mut response = HttpResponse::build(StatusCode::OK);
    response.set_header(header::CONTENT_TYPE, "text/html; charset=utf-8");
    response.body(AUTOCOMPLETE_SANDBOX_HTML)
}

const AUTOCOMPLETE_SANDBOX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Autocomplete Sandbox</title>
  <style>
    :root {
      --bg: #f3efe4;
      --panel: rgba(255, 250, 242, 0.92);
      --ink: #182126;
      --muted: #5f6b72;
      --accent: #bd4f2f;
      --accent-soft: #f1c5a9;
      --line: rgba(24, 33, 38, 0.12);
      --shadow: 0 24px 80px rgba(68, 44, 24, 0.16);
    }

    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: Georgia, "Times New Roman", serif;
      color: var(--ink);
      background:
        radial-gradient(circle at top left, rgba(189, 79, 47, 0.18), transparent 35%),
        radial-gradient(circle at bottom right, rgba(32, 129, 111, 0.18), transparent 40%),
        linear-gradient(180deg, #f7f2e9, var(--bg));
      min-height: 100vh;
    }

    main {
      max-width: 960px;
      margin: 0 auto;
      padding: 48px 20px 80px;
    }

    .card {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 28px;
      box-shadow: var(--shadow);
      overflow: hidden;
      backdrop-filter: blur(14px);
    }

    .hero {
      padding: 28px 28px 14px;
      border-bottom: 1px solid var(--line);
    }

    h1 {
      margin: 0 0 10px;
      font-size: clamp(2rem, 5vw, 4rem);
      line-height: 0.96;
      letter-spacing: -0.05em;
      text-transform: uppercase;
    }

    .hero p {
      margin: 0;
      max-width: 56ch;
      color: var(--muted);
      font-size: 1rem;
      line-height: 1.5;
    }

    .controls {
      display: grid;
      grid-template-columns: 1.5fr 1fr 180px;
      gap: 14px;
      padding: 24px 28px;
    }

    label {
      display: block;
      margin-bottom: 8px;
      font-size: 0.78rem;
      font-weight: 700;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      color: var(--muted);
    }

    input {
      width: 100%;
      border: 1px solid rgba(24, 33, 38, 0.15);
      border-radius: 16px;
      background: rgba(255, 255, 255, 0.75);
      color: var(--ink);
      padding: 15px 16px;
      font: inherit;
      outline: none;
      transition: border-color 120ms ease, transform 120ms ease, box-shadow 120ms ease;
    }

    input:focus {
      border-color: var(--accent);
      transform: translateY(-1px);
      box-shadow: 0 0 0 4px rgba(189, 79, 47, 0.12);
    }

    .meta {
      display: flex;
      gap: 18px;
      flex-wrap: wrap;
      padding: 0 28px 18px;
      color: var(--muted);
      font-size: 0.92rem;
    }

    .meta strong { color: var(--ink); }

    .results {
      padding: 0 18px 18px;
    }

    .result {
      margin: 10px 0;
      padding: 16px 18px;
      border-radius: 18px;
      background: rgba(255, 255, 255, 0.72);
      border: 1px solid rgba(24, 33, 38, 0.08);
      animation: rise 180ms ease;
    }

    .result strong {
      display: block;
      font-size: 1.05rem;
      margin-bottom: 4px;
    }

    .result small {
      color: var(--muted);
      letter-spacing: 0.04em;
      text-transform: uppercase;
    }

    .empty, .error {
      margin: 12px 10px 0;
      padding: 18px;
      border-radius: 18px;
    }

    .empty {
      background: rgba(255, 255, 255, 0.65);
      color: var(--muted);
    }

    .error {
      background: rgba(189, 79, 47, 0.12);
      color: #8a2818;
    }

    @keyframes rise {
      from { opacity: 0; transform: translateY(8px); }
      to { opacity: 1; transform: translateY(0); }
    }

    @media (max-width: 760px) {
      .controls {
        grid-template-columns: 1fr;
      }

      main {
        padding: 20px 12px 40px;
      }

      .hero, .controls, .meta {
        padding-left: 18px;
        padding-right: 18px;
      }
    }
  </style>
</head>
<body>
  <main>
    <section class="card">
      <div class="hero">
        <h1>Street Prefix Sandbox</h1>
        <p>Type the street prefix exactly as a user would. The page sends the full current prefix with the same session ID, while the backend narrows from the previously filtered in-memory set for that session whenever the prefix extends.</p>
      </div>
      <div class="controls">
        <div>
          <label for="query">Street Prefix</label>
          <input id="query" autocomplete="off" spellcheck="false" placeholder="aven..." />
        </div>
        <div>
          <label for="countryBias">Country Bias</label>
          <input id="countryBias" autocomplete="off" spellcheck="false" placeholder="FR" />
        </div>
        <div>
          <label for="sessionId">Session ID</label>
          <input id="sessionId" autocomplete="off" spellcheck="false" />
        </div>
      </div>
      <div class="meta">
        <div>Resolved query: <strong id="resolvedQuery">-</strong></div>
        <div>Matches returned: <strong id="matchCount">0</strong></div>
        <div>Status: <strong id="status">idle</strong></div>
      </div>
      <div class="results" id="results">
        <div class="empty">Start typing to call <code>POST /autocomplete</code>.</div>
      </div>
    </section>
  </main>
  <script>
    const queryEl = document.getElementById("query");
    const countryEl = document.getElementById("countryBias");
    const sessionEl = document.getElementById("sessionId");
    const resultsEl = document.getElementById("results");
    const resolvedQueryEl = document.getElementById("resolvedQuery");
    const matchCountEl = document.getElementById("matchCount");
    const statusEl = document.getElementById("status");

    sessionEl.value = crypto.randomUUID();

    let debounceTimer = null;
    let requestCounter = 0;

    queryEl.addEventListener("input", scheduleFetch);
    countryEl.addEventListener("input", scheduleFetch);
    sessionEl.addEventListener("change", scheduleFetch);

    function scheduleFetch() {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(runFetch, 120);
    }

    async function runFetch() {
      const query = queryEl.value;
      const countryBias = countryEl.value.trim();

      if (!query.trim()) {
        resolvedQueryEl.textContent = "-";
        matchCountEl.textContent = "0";
        statusEl.textContent = "idle";
        resultsEl.innerHTML = '<div class="empty">Start typing to call <code>POST /autocomplete</code>.</div>';
        return;
      }

      const currentRequest = ++requestCounter;
      statusEl.textContent = "loading";

      try {
        const response = await fetch("/autocomplete", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            session_id: sessionEl.value.trim(),
            query,
            country_bias: countryBias || null
          })
        });

        const payload = await response.json();
        if (currentRequest !== requestCounter) {
          return;
        }

        if (!response.ok) {
          throw new Error(payload?.error?.message || "request failed");
        }

        resolvedQueryEl.textContent = payload.query || "-";
        matchCountEl.textContent = String(payload.suggestions.length);
        statusEl.textContent = "ok";

        if (!payload.suggestions.length) {
          resultsEl.innerHTML = '<div class="empty">No suggestions for this prefix.</div>';
          return;
        }

        resultsEl.innerHTML = payload.suggestions.map((item) => `
          <article class="result">
            <small>${escapeHtml(item.country_code)}${item.locality ? " • " + escapeHtml(item.locality) : ""}</small>
            <strong>${escapeHtml(item.street)}</strong>
            <div>${escapeHtml(item.formatted)}</div>
          </article>
        `).join("");
      } catch (error) {
        if (currentRequest !== requestCounter) {
          return;
        }

        statusEl.textContent = "error";
        resultsEl.innerHTML = `<div class="error">${escapeHtml(error.message || "request failed")}</div>`;
      }
    }

    function escapeHtml(value) {
      return String(value)
        .replaceAll("&", "&amp;")
        .replaceAll("<", "&lt;")
        .replaceAll(">", "&gt;")
        .replaceAll('"', "&quot;")
        .replaceAll("'", "&#39;");
    }
  </script>
</body>
</html>
"#;
