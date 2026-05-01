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
    cfg.service(index);
    cfg.service(autocomplete_sandbox);
    cfg.service(resolve_address_sandbox);
}

#[web::get("/")]
async fn index(state: web::types::State<AppState>) -> HttpResponse {
    let mut response = HttpResponse::build(StatusCode::OK);
    response.set_header(header::CONTENT_TYPE, "text/html; charset=utf-8");
    response.body(render_sandbox_html(&state.addresses.country_codes()))
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
async fn autocomplete_sandbox(state: web::types::State<AppState>) -> HttpResponse {
    let mut response = HttpResponse::build(StatusCode::OK);
    response.set_header(header::CONTENT_TYPE, "text/html; charset=utf-8");
    response.body(render_sandbox_html(&state.addresses.country_codes()))
}

#[web::get("/sandbox/address-resolve")]
async fn resolve_address_sandbox(state: web::types::State<AppState>) -> HttpResponse {
    let mut response = HttpResponse::build(StatusCode::OK);
    response.set_header(header::CONTENT_TYPE, "text/html; charset=utf-8");
    response.body(render_sandbox_html(&state.addresses.country_codes()))
}

fn render_sandbox_html(country_codes: &[String]) -> String {
    SANDBOX_HTML.replace(
        "<!-- COUNTRY_BIAS_OPTIONS -->",
        &render_country_bias_options(country_codes),
    )
}

fn render_country_bias_options(country_codes: &[String]) -> String {
    country_codes
        .iter()
        .map(|code| format!(r#"<option value="{code}"></option>"#))
        .collect::<Vec<_>>()
        .join("")
}

const SANDBOX_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>AddressWise API</title>
  <style>
    :root {
      --bg: #f4efe8;
      --bg-strong: #e8dccb;
      --panel: rgba(255, 252, 247, 0.84);
      --panel-strong: rgba(255, 248, 238, 0.94);
      --ink: #172026;
      --muted: #5a676d;
      --accent: #b24b2a;
      --accent-strong: #8f3519;
      --accent-soft: #f6d6bf;
      --forest: #234d43;
      --line: rgba(23, 32, 38, 0.1);
      --shadow: 0 30px 90px rgba(69, 41, 18, 0.15);
      --radius-xl: 32px;
      --radius-lg: 24px;
      --radius-md: 18px;
    }

    * { box-sizing: border-box; }
    html { scroll-behavior: smooth; }

    body {
      margin: 0;
      font-family: Georgia, "Times New Roman", serif;
      color: var(--ink);
      background:
        radial-gradient(circle at top left, rgba(178, 75, 42, 0.18), transparent 34%),
        radial-gradient(circle at top right, rgba(35, 77, 67, 0.15), transparent 30%),
        radial-gradient(circle at bottom left, rgba(218, 160, 109, 0.16), transparent 28%),
        linear-gradient(180deg, #faf5ee, var(--bg));
      min-height: 100vh;
    }

    main {
      max-width: 1160px;
      margin: 0 auto;
      padding: 24px 20px 88px;
    }

    .shell {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: var(--radius-xl);
      box-shadow: var(--shadow);
      overflow: hidden;
      backdrop-filter: blur(16px);
    }

    .topbar {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 20px;
      padding: 18px 28px;
      border-bottom: 1px solid var(--line);
    }

    .brand {
      display: flex;
      align-items: center;
      gap: 12px;
      font-size: 0.95rem;
      letter-spacing: 0.18em;
      text-transform: uppercase;
    }

    .brand-mark {
      width: 42px;
      height: 42px;
      display: grid;
      place-items: center;
      border-radius: 14px;
      color: white;
      background: linear-gradient(135deg, var(--accent), var(--forest));
      font-weight: 700;
      box-shadow: inset 0 1px 0 rgba(255,255,255,0.25);
    }

    .topnav {
      display: flex;
      gap: 16px;
      flex-wrap: wrap;
      align-items: center;
    }

    .topnav a,
    .topnav button {
      border: 0;
      background: transparent;
      color: var(--muted);
      font: inherit;
      cursor: pointer;
      text-decoration: none;
      padding: 0;
    }

    .hero {
      display: grid;
      grid-template-columns: minmax(0, 1.2fr) minmax(320px, 0.8fr);
      gap: 28px;
      padding: 34px 28px 28px;
      border-bottom: 1px solid var(--line);
    }

    .eyebrow {
      display: inline-flex;
      align-items: center;
      gap: 10px;
      padding: 8px 12px;
      border-radius: 999px;
      background: rgba(255,255,255,0.62);
      border: 1px solid rgba(178, 75, 42, 0.14);
      color: var(--accent-strong);
      font-size: 0.76rem;
      letter-spacing: 0.12em;
      text-transform: uppercase;
    }

    h1 {
      margin: 16px 0 14px;
      font-size: clamp(2.8rem, 7vw, 5.5rem);
      line-height: 0.92;
      letter-spacing: -0.05em;
      text-transform: uppercase;
    }

    .hero-copy p {
      margin: 0 0 16px;
      max-width: 60ch;
      color: var(--muted);
      font-size: 1.05rem;
      line-height: 1.65;
    }

    .hero-actions {
      display: flex;
      gap: 12px;
      flex-wrap: wrap;
      margin-top: 24px;
    }

    .action,
    button.action {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 10px;
      min-height: 48px;
      padding: 0 18px;
      border-radius: 999px;
      border: 1px solid transparent;
      text-decoration: none;
      cursor: pointer;
      font: inherit;
      font-weight: 700;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }

    .action-primary {
      background: linear-gradient(135deg, var(--accent), var(--accent-strong));
      color: white;
      box-shadow: 0 18px 35px rgba(178, 75, 42, 0.24);
    }

    .action-secondary {
      background: rgba(255,255,255,0.58);
      color: var(--ink);
      border-color: rgba(23, 32, 38, 0.1);
    }

    .hero-card,
    .section-card,
    .demo-card {
      border-radius: var(--radius-lg);
      border: 1px solid var(--line);
      background: var(--panel-strong);
      box-shadow: inset 0 1px 0 rgba(255,255,255,0.45);
    }

    .hero-card {
      padding: 22px;
      background:
        linear-gradient(160deg, rgba(255,255,255,0.72), rgba(246,214,191,0.76)),
        rgba(255,248,238,0.96);
    }

    .hero-card h2,
    .section-card h2,
    .demo-head h2 {
      margin: 0 0 10px;
      font-size: 1.15rem;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }

    .signal-grid,
    .feature-grid,
    .pricing-grid,
    .trust-grid,
    .steps-grid,
    .demo-grid {
      display: grid;
      gap: 18px;
    }

    .signal-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      margin-top: 18px;
    }

    .signal {
      padding: 16px;
      border-radius: var(--radius-md);
      background: rgba(255,255,255,0.7);
      border: 1px solid rgba(23, 32, 38, 0.08);
    }

    .signal strong {
      display: block;
      font-size: 1.7rem;
      letter-spacing: -0.04em;
      margin-bottom: 4px;
    }

    .stack {
      display: grid;
      gap: 18px;
      padding: 22px 28px 28px;
    }

    .section-card {
      padding: 22px;
    }

    .section-card p,
    .demo-head p,
    .pricing-card p,
    .step p,
    .small-copy {
      margin: 0;
      color: var(--muted);
      line-height: 1.6;
    }

    .feature-grid,
    .trust-grid,
    .steps-grid,
    .pricing-grid {
      grid-template-columns: repeat(3, minmax(0, 1fr));
      margin-top: 18px;
    }

    .feature,
    .trust,
    .step,
    .pricing-card {
      padding: 18px;
      border-radius: var(--radius-md);
      background: rgba(255,255,255,0.7);
      border: 1px solid rgba(23, 32, 38, 0.08);
    }

    .feature strong,
    .trust strong,
    .step strong,
    .pricing-card strong {
      display: block;
      margin-bottom: 8px;
      font-size: 1.02rem;
    }

    .pricing-card.featured {
      background: linear-gradient(180deg, rgba(246,214,191,0.72), rgba(255,255,255,0.78));
      border-color: rgba(178, 75, 42, 0.24);
      transform: translateY(-4px);
    }

    .plan-kicker {
      color: var(--accent-strong);
      font-size: 0.76rem;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      margin-bottom: 8px;
    }

    .price {
      display: flex;
      align-items: baseline;
      gap: 10px;
      margin: 6px 0 10px;
    }

    .price .value {
      font-size: 2.35rem;
      letter-spacing: -0.05em;
    }

    .pricing-card ul,
    .step ul {
      margin: 14px 0 0;
      padding-left: 18px;
      color: var(--muted);
      line-height: 1.6;
    }

    .demo-card {
      overflow: hidden;
    }

    .demo-head {
      padding: 22px 22px 8px;
      border-bottom: 1px solid var(--line);
    }

    .demo-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      padding: 18px;
    }

    .sandbox {
      border: 1px solid var(--line);
      border-radius: 22px;
      background: rgba(255, 255, 255, 0.58);
      overflow: hidden;
    }

    .sandbox-head {
      padding: 20px 22px 8px;
    }

    .sandbox-head h3 {
      margin: 0 0 6px;
      font-size: 1.05rem;
      letter-spacing: 0.05em;
      text-transform: uppercase;
    }

    .sandbox-head p {
      margin: 0;
      color: var(--muted);
      line-height: 1.55;
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
      font-size: 0.74rem;
      font-weight: 700;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      color: var(--muted);
    }

    input {
      width: 100%;
      border: 1px solid rgba(23, 32, 38, 0.14);
      border-radius: 16px;
      background: rgba(255, 255, 255, 0.84);
      color: var(--ink);
      padding: 15px 16px;
      font: inherit;
      outline: none;
      transition: border-color 120ms ease, transform 120ms ease, box-shadow 120ms ease;
    }

    input:focus {
      border-color: var(--accent);
      transform: translateY(-1px);
      box-shadow: 0 0 0 4px rgba(178, 75, 42, 0.12);
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
      background: rgba(255, 255, 255, 0.78);
      border: 1px solid rgba(23, 32, 38, 0.08);
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
      background: rgba(255, 255, 255, 0.68);
      color: var(--muted);
    }

    .error {
      background: rgba(178, 75, 42, 0.12);
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
      background: rgba(255, 255, 255, 0.78);
      border: 1px solid rgba(23, 32, 38, 0.08);
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

    .code-strip {
      margin-top: 18px;
      padding: 18px;
      border-radius: var(--radius-md);
      background: #172026;
      color: #f4efe8;
      font-family: "Courier New", monospace;
      overflow: auto;
    }

    .footer-cta {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 18px;
      align-items: center;
      padding: 22px;
      border-radius: var(--radius-lg);
      background: linear-gradient(135deg, rgba(35,77,67,0.92), rgba(23,32,38,0.94));
      color: white;
    }

    .footer-cta p {
      margin: 0;
      color: rgba(255,255,255,0.78);
      line-height: 1.6;
    }

    .footer-cta strong {
      display: block;
      font-size: 1.45rem;
      margin-bottom: 8px;
    }

    code {
      font-family: "Courier New", monospace;
    }

    @keyframes rise {
      from { opacity: 0; transform: translateY(8px); }
      to { opacity: 1; transform: translateY(0); }
    }

    @media (max-width: 960px) {
      .hero,
      .demo-grid,
      .feature-grid,
      .pricing-grid,
      .trust-grid,
      .steps-grid,
      .footer-cta {
        grid-template-columns: 1fr;
      }
    }

    @media (max-width: 760px) {
      .topbar,
      .controls,
      .resolve-controls,
      .detail-row,
      .signal-grid {
        grid-template-columns: 1fr;
      }

      main {
        padding: 12px 12px 40px;
      }

      .topbar,
      .hero,
      .stack,
      .controls,
      .meta,
      .demo-head {
        padding-left: 18px;
        padding-right: 18px;
      }
    }
  </style>
</head>
<body>
  <main>
    <section class="shell">
      <div class="topbar">
        <div class="brand">
          <div class="brand-mark">AW</div>
          <div>AddressWise API</div>
        </div>
        <nav class="topnav">
          <a href="#product">Product</a>
          <a href="#pricing">Pricing</a>
          <a href="#billing">Billing</a>
          <a href="#demo">Live Demo</a>
        </nav>
      </div>

      <section class="hero">
        <div class="hero-copy">
          <div class="eyebrow">Address intelligence for forms, checkout, CRM, and logistics</div>
          <h1>Sell fewer failed deliveries.</h1>
          <p>AddressWise gives product teams one API for street autocomplete and full address resolution across real postal datasets. You can test the live endpoints on this page, then move straight into production with API keys and usage-based billing.</p>
          <p>The strongest fit is B2B infrastructure: checkout forms, onboarding flows, shipping validation, lead capture, marketplace seller forms, and back-office cleanup.</p>
          <div class="hero-actions">
            <a class="action action-primary" href="#demo">Try the live demo</a>
            <a class="action action-secondary" href="#pricing">See pricing</a>
          </div>
        </div>
        <aside class="hero-card">
          <h2>What you are buying</h2>
          <p class="small-copy">A hosted API, not a static dataset. Your team gets address autocomplete, full-text resolution, country-aware matching, and a clear path from trial to production.</p>
          <div class="signal-grid">
            <div class="signal">
              <strong>API-first</strong>
              <span>Autocomplete and resolve endpoints for web, mobile, and backend flows.</span>
            </div>
            <div class="signal">
              <strong>Usage-based</strong>
              <span>Pay monthly for included request credits, then overages as you scale.</span>
            </div>
            <div class="signal">
              <strong>Global biasing</strong>
              <span>Country-aware matching and country-biased autocomplete from one integration.</span>
            </div>
            <div class="signal">
              <strong>Fast evaluation</strong>
              <span>Sales page and sandbox stay together so buyers can validate behavior immediately.</span>
            </div>
          </div>
        </aside>
      </section>

      <div class="stack">
        <section class="section-card" id="product">
          <h2>What AddressWise does</h2>
          <p>Use one service to suggest candidate streets while a user types, then resolve messy free-text or structured inputs into a normalized address record with diagnostics and scoring.</p>
          <div class="feature-grid">
            <article class="feature">
              <strong>Street autocomplete</strong>
              <p>Session-aware narrowing for fast prefix matching, with optional country bias to keep results relevant in international products.</p>
            </article>
            <article class="feature">
              <strong>Free-text resolution</strong>
              <p>Accepts messy user input such as postal code, street typo, and house number in one line, then ranks the best candidate.</p>
            </article>
            <article class="feature">
              <strong>Structured fallback</strong>
              <p>Teams that already split fields can send street, house number, city, postal code, and country separately for a stronger match.</p>
            </article>
          </div>
        </section>

        <section class="section-card" id="pricing">
          <h2>Pricing that maps to API usage</h2>
          <p>Keep the entry path simple: monthly plans include a request budget, one or more API keys, and overage billing. Large customers move to contract and enterprise support.</p>
          <div class="pricing-grid">
            <article class="pricing-card">
              <div class="plan-kicker">Starter</div>
              <strong>Evaluate and prototype</strong>
              <div class="price"><span class="value">EUR 99</span><span>/ month</span></div>
              <p>Up to 100k requests per month across both endpoints.</p>
              <ul>
                <li>2 API keys</li>
                <li>Basic analytics</li>
                <li>Email support</li>
                <li>Overage at EUR 1.20 per extra 1k requests</li>
              </ul>
            </article>
            <article class="pricing-card featured">
              <div class="plan-kicker">Growth</div>
              <strong>Production for serious volume</strong>
              <div class="price"><span class="value">EUR 499</span><span>/ month</span></div>
              <p>Up to 1M requests per month with operational headroom.</p>
              <ul>
                <li>10 API keys</li>
                <li>Priority email support</li>
                <li>Usage alerts and billing controls</li>
                <li>Overage at EUR 0.75 per extra 1k requests</li>
              </ul>
            </article>
            <article class="pricing-card">
              <div class="plan-kicker">Enterprise</div>
              <strong>Custom deployment and SLAs</strong>
              <div class="price"><span class="value">Custom</span><span>contract</span></div>
              <p>For logistics, marketplaces, and large multi-country integrations.</p>
              <ul>
                <li>Dedicated support and SLAs</li>
                <li>Private networking or on-prem options</li>
                <li>Custom country coverage and refresh cadence</li>
                <li>Batch cleansing and migration support</li>
              </ul>
            </article>
          </div>
        </section>

        <section class="section-card" id="billing">
          <h2>How paying works</h2>
          <p>The simplest mechanism is API keys plus monthly request credits. Each workspace gets one billing account and one or more keys. Every request burns a credit, and overages are billed automatically at the plan rate.</p>
          <div class="steps-grid">
            <article class="step">
              <strong>1. Create workspace</strong>
              <p>Your company creates an account, adds a card or invoice billing profile, and picks a monthly plan.</p>
            </article>
            <article class="step">
              <strong>2. Issue API keys</strong>
              <p>Create separate keys for production, staging, and sandbox use so you can isolate usage and rotate keys safely.</p>
            </article>
            <article class="step">
              <strong>3. Consume credits</strong>
              <p>Each call to <code>POST /autocomplete</code> or <code>POST /resolve-address</code> consumes credits and appears in usage analytics.</p>
            </article>
          </div>
          <div class="code-strip">Authorization: Bearer aw_live_xxxxxxxxxxxxx

POST /autocomplete
POST /resolve-address

Monthly plan includes request credits.
When credits are exhausted, requests continue and overages are billed automatically.</div>
        </section>

        <section class="section-card">
          <h2>What convinces a buyer</h2>
          <p>Infrastructure products close faster when the value is obvious in a live demo. Keep this page public enough for prospects to test behavior, while the commercial model stays clear and operational.</p>
          <div class="trust-grid">
            <article class="trust">
              <strong>Fewer abandoned forms</strong>
              <p>Autocomplete reduces friction in checkout and onboarding flows.</p>
            </article>
            <article class="trust">
              <strong>Better delivery quality</strong>
              <p>Resolution catches messy inputs before they hit downstream shipping systems.</p>
            </article>
            <article class="trust">
              <strong>Clear buying path</strong>
              <p>Trial on the live page, deploy with keys, upgrade by request volume.</p>
            </article>
          </div>
        </section>

        <section class="demo-card" id="demo">
          <div class="demo-head">
            <h2>Live product demo</h2>
            <p>This is the same sales page your prospects can use to validate the product. The forms below call the live API directly, so the marketing story and the technical proof stay in one place.</p>
          </div>
          <div class="demo-grid">
            <section class="sandbox">
              <div class="sandbox-head">
                <h3>Street autocomplete</h3>
                <p>Type a street prefix exactly as a user would. Country bias now comes from countries loaded in the database, so buyers can test realistic scope without guessing codes.</p>
              </div>
              <div class="controls">
                <div>
                  <label for="autocompleteQuery">Street Prefix</label>
                  <input id="autocompleteQuery" autocomplete="off" spellcheck="false" placeholder="aven..." />
                </div>
                <div>
                  <label for="countryBias">Country Bias</label>
                  <input id="countryBias" list="countryBiasOptions" autocomplete="off" spellcheck="false" placeholder="FR" />
                  <datalist id="countryBiasOptions"><!-- COUNTRY_BIAS_OPTIONS --></datalist>
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
                <h3>Address resolve</h3>
                <p>Send a free-text query, structured fields, or both. The live response shows the selected address and matching diagnostics so a prospect can judge quality quickly.</p>
              </div>
              <div class="controls resolve-controls">
                <div>
                  <label for="resolveQuery">Query</label>
                  <input id="resolveQuery" autocomplete="off" spellcheck="false" placeholder="hlava ulica 47 vojany 07672 sk" />
                </div>
                <div>
                  <label for="resolveCountry">Country</label>
                  <input id="resolveCountry" autocomplete="off" spellcheck="false" placeholder="SK" />
                </div>
                <div>
                  <label for="resolveStreet">Street</label>
                  <input id="resolveStreet" autocomplete="off" spellcheck="false" placeholder="Hlavna ulica" />
                </div>
                <div>
                  <label for="resolveHouseNumber">House Number</label>
                  <input id="resolveHouseNumber" autocomplete="off" spellcheck="false" placeholder="47" />
                </div>
                <div>
                  <label for="resolveCity">City</label>
                  <input id="resolveCity" autocomplete="off" spellcheck="false" placeholder="Vojany" />
                </div>
                <div>
                  <label for="resolvePostalCode">Postal Code</label>
                  <input id="resolvePostalCode" autocomplete="off" spellcheck="false" placeholder="07672" />
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
                  <button id="resolveButton" type="button" class="action action-primary" style="width: 100%;">Resolve Address</button>
                </div>
              </div>
            </section>
          </div>
        </section>

        <section class="footer-cta">
          <div>
            <strong>Ready to sell this as infrastructure</strong>
            <p>Keep this page as the public product surface: live demo, pricing, billing model, and technical proof in one place.</p>
          </div>
          <a class="action action-primary" href="#demo">Run the demo</a>
        </section>
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
"##;
