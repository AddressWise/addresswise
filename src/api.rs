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
    cfg.service(resolve_address_sandbox);
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
    response.body(SANDBOX_HTML)
}

#[web::get("/sandbox/address-resolve")]
async fn resolve_address_sandbox() -> HttpResponse {
    let mut response = HttpResponse::build(StatusCode::OK);
    response.set_header(header::CONTENT_TYPE, "text/html; charset=utf-8");
    response.body(SANDBOX_HTML)
}

const SANDBOX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Address Sandbox</title>
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

    .stack {
      display: grid;
      gap: 18px;
      padding: 20px 28px 28px;
    }

    .sandbox {
      border: 1px solid var(--line);
      border-radius: 22px;
      background: rgba(255, 255, 255, 0.5);
      overflow: hidden;
    }

    .sandbox-head {
      padding: 20px 22px 8px;
    }

    .sandbox-head h2 {
      margin: 0 0 6px;
      font-size: 1.15rem;
      letter-spacing: 0.02em;
      text-transform: uppercase;
    }

    .sandbox-head p {
      margin: 0;
      color: var(--muted);
      line-height: 1.5;
    }

    .controls {
      display: grid;
      grid-template-columns: 1.5fr 1fr 180px;
      gap: 14px;
      padding: 20px 22px;
    }

    .resolve-controls {
      grid-template-columns: repeat(2, minmax(0, 1fr));
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
      padding: 0 22px 18px;
      color: var(--muted);
      font-size: 0.92rem;
    }

    .meta strong { color: var(--ink); }

    .results {
      padding: 0 12px 12px;
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

    .details {
      display: grid;
      gap: 10px;
    }

    .detail-row {
      display: grid;
      grid-template-columns: 150px 1fr;
      gap: 12px;
      padding: 12px 14px;
      border-radius: 16px;
      background: rgba(255, 255, 255, 0.72);
      border: 1px solid rgba(24, 33, 38, 0.08);
    }

    .detail-row strong {
      text-transform: uppercase;
      letter-spacing: 0.08em;
      font-size: 0.75rem;
      color: var(--muted);
    }

    .detail-row span {
      word-break: break-word;
    }

    @keyframes rise {
      from { opacity: 0; transform: translateY(8px); }
      to { opacity: 1; transform: translateY(0); }
    }

    @media (max-width: 760px) {
      .controls,
      .resolve-controls,
      .detail-row {
        grid-template-columns: 1fr;
      }

      main {
        padding: 20px 12px 40px;
      }

      .hero, .stack, .controls, .meta {
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
        <h1>Address Sandbox</h1>
        <p>Use the same page to probe street autocomplete and full address resolution. Both panels call the live API endpoints directly so you can test behavior without leaving the sandbox.</p>
      </div>
      <div class="stack">
        <section class="sandbox">
          <div class="sandbox-head">
            <h2>Street Autocomplete</h2>
            <p>Type the street prefix exactly as a user would. The page sends the full current prefix with the same session ID, while the backend narrows from the previously filtered in-memory set for that session whenever the prefix extends.</p>
          </div>
          <div class="controls">
            <div>
              <label for="autocompleteQuery">Street Prefix</label>
              <input id="autocompleteQuery" autocomplete="off" spellcheck="false" placeholder="aven..." />
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
            <div>Status: <strong id="autocompleteStatus">idle</strong></div>
          </div>
          <div class="results" id="autocompleteResults">
            <div class="empty">Start typing to call <code>POST /autocomplete</code>.</div>
          </div>
        </section>
        <section class="sandbox">
          <div class="sandbox-head">
            <h2>Address Resolve</h2>
            <p>Send either a free-text query, structured fields, or both. The sandbox posts directly to <code>POST /resolve-address</code> and shows the selected match with diagnostics.</p>
          </div>
          <div class="controls resolve-controls">
            <div>
              <label for="resolveQuery">Query</label>
              <input id="resolveQuery" autocomplete="off" spellcheck="false" placeholder="avenue de france 123 stiring wendel 57350 fr" />
            </div>
            <div>
              <label for="resolveCountry">Country</label>
              <input id="resolveCountry" autocomplete="off" spellcheck="false" placeholder="FR" />
            </div>
            <div>
              <label for="resolveStreet">Street</label>
              <input id="resolveStreet" autocomplete="off" spellcheck="false" placeholder="Avenue de France" />
            </div>
            <div>
              <label for="resolveHouseNumber">House Number</label>
              <input id="resolveHouseNumber" autocomplete="off" spellcheck="false" placeholder="123" />
            </div>
            <div>
              <label for="resolveCity">City</label>
              <input id="resolveCity" autocomplete="off" spellcheck="false" placeholder="Stiring-Wendel" />
            </div>
            <div>
              <label for="resolvePostalCode">Postal Code</label>
              <input id="resolvePostalCode" autocomplete="off" spellcheck="false" placeholder="57350" />
            </div>
          </div>
          <div class="meta">
            <div>Chosen query: <strong id="resolveResolvedQuery">-</strong></div>
            <div>Score: <strong id="resolveScore">-</strong></div>
            <div>Status: <strong id="resolveStatus">idle</strong></div>
          </div>
          <div class="results" id="resolveResults">
            <div class="empty">Enter a query or structured address, then click Resolve.</div>
          </div>
          <div class="controls" style="padding-top: 0;">
            <div>
              <button id="resolveButton" type="button" style="width: 100%; border: 0; border-radius: 16px; padding: 15px 16px; font: inherit; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; color: white; background: var(--accent); cursor: pointer;">Resolve Address</button>
            </div>
          </div>
        </div>
      </div>
    </section>
  </main>
  <script>
    const queryEl = document.getElementById("autocompleteQuery");
    const countryEl = document.getElementById("countryBias");
    const sessionEl = document.getElementById("sessionId");
    const autocompleteResultsEl = document.getElementById("autocompleteResults");
    const resolvedQueryEl = document.getElementById("resolvedQuery");
    const matchCountEl = document.getElementById("matchCount");
    const autocompleteStatusEl = document.getElementById("autocompleteStatus");
    const resolveQueryEl = document.getElementById("resolveQuery");
    const resolveCountryEl = document.getElementById("resolveCountry");
    const resolveStreetEl = document.getElementById("resolveStreet");
    const resolveHouseNumberEl = document.getElementById("resolveHouseNumber");
    const resolveCityEl = document.getElementById("resolveCity");
    const resolvePostalCodeEl = document.getElementById("resolvePostalCode");
    const resolveButtonEl = document.getElementById("resolveButton");
    const resolveResolvedQueryEl = document.getElementById("resolveResolvedQuery");
    const resolveScoreEl = document.getElementById("resolveScore");
    const resolveStatusEl = document.getElementById("resolveStatus");
    const resolveResultsEl = document.getElementById("resolveResults");

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
        autocompleteStatusEl.textContent = "idle";
        autocompleteResultsEl.innerHTML = '<div class="empty">Start typing to call <code>POST /autocomplete</code>.</div>';
        return;
      }

      const currentRequest = ++requestCounter;
      autocompleteStatusEl.textContent = "loading";

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
        autocompleteStatusEl.textContent = "ok";

        if (!payload.suggestions.length) {
          autocompleteResultsEl.innerHTML = '<div class="empty">No suggestions for this prefix.</div>';
          return;
        }

        autocompleteResultsEl.innerHTML = payload.suggestions.map((item) => `
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

        autocompleteStatusEl.textContent = "error";
        autocompleteResultsEl.innerHTML = `<div class="error">${escapeHtml(error.message || "request failed")}</div>`;
      }
    }

    resolveButtonEl.addEventListener("click", runResolveFetch);
    [
      resolveQueryEl,
      resolveCountryEl,
      resolveStreetEl,
      resolveHouseNumberEl,
      resolveCityEl,
      resolvePostalCodeEl
    ].forEach((el) => {
      el.addEventListener("keydown", (event) => {
        if (event.key === "Enter") {
          event.preventDefault();
          runResolveFetch();
        }
      });
    });

    async function runResolveFetch() {
      const payload = {
        query: emptyToNull(resolveQueryEl.value),
        street: emptyToNull(resolveStreetEl.value),
        house_number: emptyToNull(resolveHouseNumberEl.value),
        city: emptyToNull(resolveCityEl.value),
        postal_code: emptyToNull(resolvePostalCodeEl.value),
        country: emptyToNull(resolveCountryEl.value)
      };

      if (!Object.values(payload).some(Boolean)) {
        resolveResolvedQueryEl.textContent = "-";
        resolveScoreEl.textContent = "-";
        resolveStatusEl.textContent = "idle";
        resolveResultsEl.innerHTML = '<div class="empty">Enter a query or structured address, then click Resolve.</div>';
        return;
      }

      resolveStatusEl.textContent = "loading";
      resolveResultsEl.innerHTML = '<div class="empty">Resolving address...</div>';

      try {
        const response = await fetch("/resolve-address", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload)
        });

        const body = await response.json();
        if (!response.ok) {
          throw new Error(body?.error?.message || "request failed");
        }

        resolveResolvedQueryEl.textContent = body.query || "-";
        resolveScoreEl.textContent = typeof body.score === "number" ? body.score.toFixed(4) : "-";
        resolveStatusEl.textContent = "ok";
        resolveResultsEl.innerHTML = renderResolveResult(body);
      } catch (error) {
        resolveStatusEl.textContent = "error";
        resolveResultsEl.innerHTML = `<div class="error">${escapeHtml(error.message || "request failed")}</div>`;
      }
    }

    function renderResolveResult(payload) {
      const address = payload.address || {};
      const diagnostics = payload.diagnostics || {};
      const rows = [
        ["Formatted", address.formatted],
        ["Country", address.country_code],
        ["Admin Area", address.admin_area],
        ["Locality", address.locality],
        ["Dependent Locality", address.dependent_locality],
        ["Street", address.street],
        ["House Number", address.house_number],
        ["House Number Type", address.house_number_type],
        ["Unit", address.unit],
        ["Postal Code", address.postal_code],
        ["Latitude", address.latitude],
        ["Longitude", address.longitude],
        ["Address ID", address.id],
        ["Trigram Score", diagnostics.trigram_score],
        ["Levenshtein Distance", diagnostics.levenshtein_distance]
      ].filter(([, value]) => value !== null && value !== undefined && String(value) !== "");

      return `
        <article class="result">
          <strong>${escapeHtml(address.formatted || "Resolved Address")}</strong>
          <small>${escapeHtml(address.country_code || "-")}${address.locality ? " • " + escapeHtml(address.locality) : ""}</small>
          <div class="details">
            ${rows.map(([label, value]) => `
              <div class="detail-row">
                <strong>${escapeHtml(label)}</strong>
                <span>${escapeHtml(value)}</span>
              </div>
            `).join("")}
          </div>
        </article>
      `;
    }

    function emptyToNull(value) {
      const trimmed = value.trim();
      return trimmed ? trimmed : null;
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
