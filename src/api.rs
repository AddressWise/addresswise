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
  <title>AddressWise</title>
  <style>
    :root {
      --page: #f5f9ff;
      --page-strong: #eef5ff;
      --panel: rgba(255, 255, 255, 0.88);
      --panel-solid: #ffffff;
      --line: rgba(34, 64, 112, 0.12);
      --line-strong: rgba(34, 64, 112, 0.18);
      --ink: #11233f;
      --muted: #60718d;
      --soft: #8da0bf;
      --accent: #1565f7;
      --accent-strong: #0f4fcc;
      --accent-soft: rgba(21, 101, 247, 0.1);
      --success: #0d8a6a;
      --shadow: 0 26px 70px rgba(38, 78, 153, 0.12);
      --shadow-soft: 0 18px 42px rgba(38, 78, 153, 0.08);
      --radius-xl: 32px;
      --radius-lg: 24px;
      --radius-md: 18px;
      --radius-sm: 14px;
      --content: 1180px;
    }

    * { box-sizing: border-box; }
    html { scroll-behavior: smooth; }

    body {
      margin: 0;
      font-family: "Avenir Next", "Segoe UI", "Helvetica Neue", sans-serif;
      color: var(--ink);
      background:
        radial-gradient(circle at top left, rgba(21, 101, 247, 0.12), transparent 32%),
        radial-gradient(circle at 85% 12%, rgba(111, 179, 255, 0.18), transparent 24%),
        linear-gradient(180deg, #fbfdff 0%, var(--page) 55%, #ffffff 100%);
      min-height: 100vh;
    }

    a {
      color: inherit;
      text-decoration: none;
    }

    code {
      font-family: "SFMono-Regular", "Consolas", "Liberation Mono", monospace;
    }

    main {
      width: min(var(--content), calc(100% - 32px));
      margin: 0 auto;
      padding: 24px 0 88px;
    }

    .page-shell {
      border: 1px solid rgba(255, 255, 255, 0.78);
      border-radius: 36px;
      background: linear-gradient(180deg, rgba(255, 255, 255, 0.88), rgba(255, 255, 255, 0.72));
      box-shadow: var(--shadow);
      overflow: hidden;
      backdrop-filter: blur(24px);
    }

    .topbar {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 24px;
      padding: 22px 34px;
      border-bottom: 1px solid rgba(34, 64, 112, 0.08);
      background: rgba(255, 255, 255, 0.72);
    }

    .brand {
      display: flex;
      align-items: center;
      gap: 14px;
      font-size: 0.95rem;
      font-weight: 700;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }

    .brand-mark {
      width: 44px;
      height: 44px;
      display: grid;
      place-items: center;
      border-radius: 14px;
      background: linear-gradient(135deg, #1e6dff, #71bbff);
      color: white;
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.38);
    }

    .topnav {
      display: flex;
      align-items: center;
      gap: 22px;
      flex-wrap: wrap;
      color: var(--muted);
      font-size: 0.95rem;
    }

    .topnav a {
      transition: color 150ms ease;
    }

    .topnav a:hover {
      color: var(--ink);
    }

    .hero {
      display: grid;
      grid-template-columns: minmax(0, 1.05fr) minmax(420px, 0.95fr);
      gap: 40px;
      padding: 54px 34px 46px;
      align-items: center;
    }

    .eyebrow {
      display: inline-flex;
      align-items: center;
      gap: 10px;
      padding: 10px 14px;
      border-radius: 999px;
      background: var(--accent-soft);
      color: var(--accent-strong);
      font-size: 0.78rem;
      font-weight: 700;
      letter-spacing: 0.1em;
      text-transform: uppercase;
    }

    h1 {
      margin: 22px 0 18px;
      max-width: 11ch;
      font-size: clamp(3rem, 7vw, 5.6rem);
      line-height: 0.95;
      letter-spacing: -0.07em;
    }

    .hero-copy p {
      margin: 0;
      max-width: 60ch;
      color: var(--muted);
      font-size: 1.08rem;
      line-height: 1.75;
    }

    .hero-actions {
      display: flex;
      gap: 14px;
      flex-wrap: wrap;
      margin: 28px 0 36px;
    }

    .action,
    button.action {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-height: 52px;
      padding: 0 20px;
      border: 1px solid transparent;
      border-radius: 999px;
      cursor: pointer;
      font: inherit;
      font-weight: 700;
      transition: transform 150ms ease, box-shadow 150ms ease, border-color 150ms ease, background 150ms ease;
    }

    .action:hover,
    button.action:hover {
      transform: translateY(-1px);
    }

    .action-primary {
      background: linear-gradient(135deg, var(--accent), #4d94ff);
      color: white;
      box-shadow: 0 16px 34px rgba(21, 101, 247, 0.22);
    }

    .action-secondary {
      background: rgba(255, 255, 255, 0.84);
      color: var(--ink);
      border-color: var(--line);
    }

    .hero-metrics {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 14px;
      max-width: 620px;
    }

    .metric {
      padding: 18px;
      border-radius: 20px;
      background: rgba(255, 255, 255, 0.8);
      border: 1px solid var(--line);
      box-shadow: var(--shadow-soft);
    }

    .metric strong {
      display: block;
      margin-bottom: 6px;
      font-size: 1.35rem;
      letter-spacing: -0.04em;
    }

    .metric span {
      color: var(--muted);
      font-size: 0.95rem;
      line-height: 1.5;
    }

    .hero-visual {
      position: relative;
    }

    .hero-visual::before {
      content: "";
      position: absolute;
      inset: -20px 8% auto auto;
      width: 190px;
      height: 190px;
      border-radius: 50%;
      background: radial-gradient(circle, rgba(21, 101, 247, 0.18), transparent 68%);
      pointer-events: none;
    }

    .demo-window {
      position: relative;
      border-radius: 28px;
      border: 1px solid rgba(34, 64, 112, 0.1);
      background: rgba(255, 255, 255, 0.92);
      box-shadow: var(--shadow);
      overflow: hidden;
    }

    .demo-window-head {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
      padding: 18px 20px;
      border-bottom: 1px solid rgba(34, 64, 112, 0.08);
      background: linear-gradient(180deg, rgba(245, 249, 255, 0.95), rgba(255, 255, 255, 0.92));
    }

    .window-dots {
      display: flex;
      gap: 8px;
    }

    .window-dots span {
      width: 10px;
      height: 10px;
      border-radius: 50%;
      background: rgba(34, 64, 112, 0.18);
    }

    .demo-window-head strong {
      font-size: 0.95rem;
    }

    .demo-window-head small {
      color: var(--muted);
    }

    .typed-demo {
      padding: 22px;
      display: grid;
      gap: 18px;
      background:
        radial-gradient(circle at top right, rgba(21, 101, 247, 0.05), transparent 30%),
        white;
    }

    .product-bar {
      display: flex;
      justify-content: space-between;
      gap: 14px;
      align-items: center;
      color: var(--muted);
      font-size: 0.9rem;
    }

    .product-pill {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 8px 12px;
      border-radius: 999px;
      background: var(--accent-soft);
      color: var(--accent-strong);
      font-weight: 700;
    }

    .hero-input {
      position: relative;
      padding: 18px 20px;
      border-radius: 22px;
      border: 1px solid rgba(21, 101, 247, 0.15);
      background: linear-gradient(180deg, #ffffff, #f7fbff);
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.8);
    }

    .hero-input label,
    .field label {
      display: block;
      margin-bottom: 8px;
      color: var(--muted);
      font-size: 0.76rem;
      font-weight: 700;
      letter-spacing: 0.1em;
      text-transform: uppercase;
    }

    .typed-line {
      min-height: 34px;
      font-size: 1.32rem;
      letter-spacing: -0.03em;
      white-space: nowrap;
      overflow: hidden;
    }

    .cursor {
      display: inline-block;
      width: 1px;
      height: 1.15em;
      margin-left: 4px;
      background: var(--accent);
      vertical-align: -0.16em;
      animation: blink 1s steps(1) infinite;
    }

    .demo-results {
      display: grid;
      gap: 12px;
    }

    .demo-result {
      padding: 14px 16px;
      border-radius: 18px;
      border: 1px solid rgba(34, 64, 112, 0.08);
      background: rgba(247, 251, 255, 0.9);
      opacity: 0;
      transform: translateY(8px);
      animation: float-in 320ms ease forwards;
    }

    .demo-result.active {
      border-color: rgba(21, 101, 247, 0.22);
      background: linear-gradient(180deg, rgba(232, 242, 255, 0.95), rgba(247, 251, 255, 0.92));
    }

    .demo-result strong,
    .live-result strong,
    .resolved-card strong {
      display: block;
      margin-bottom: 4px;
      font-size: 1rem;
    }

    .demo-result small,
    .live-result small,
    .resolved-card small {
      color: var(--muted);
      font-size: 0.84rem;
      text-transform: uppercase;
      letter-spacing: 0.06em;
    }

    .demo-footnote {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 14px;
      color: var(--muted);
      font-size: 0.9rem;
    }

    .section-stack {
      display: grid;
      gap: 22px;
      padding: 0 34px 34px;
    }

    .section {
      padding: 32px;
      border: 1px solid var(--line);
      border-radius: 28px;
      background: rgba(255, 255, 255, 0.8);
      box-shadow: var(--shadow-soft);
    }

    .section-header {
      display: flex;
      justify-content: space-between;
      gap: 20px;
      align-items: end;
      margin-bottom: 22px;
    }

    .section-header h2 {
      margin: 0 0 10px;
      font-size: clamp(1.8rem, 3vw, 2.6rem);
      letter-spacing: -0.05em;
    }

    .section-header p,
    .section-copy,
    .field-note,
    .live-empty,
    .error,
    .detail-value,
    .endpoint-copy {
      margin: 0;
      color: var(--muted);
      line-height: 1.7;
    }

    .cards-3,
    .cards-2 {
      display: grid;
      gap: 16px;
    }

    .cards-3 {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }

    .cards-2 {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .card {
      padding: 22px;
      border-radius: 22px;
      border: 1px solid var(--line);
      background: linear-gradient(180deg, rgba(255, 255, 255, 0.94), rgba(247, 251, 255, 0.88));
    }

    .card strong {
      display: block;
      margin-bottom: 8px;
      font-size: 1.05rem;
    }

    .card p {
      margin: 0;
      color: var(--muted);
      line-height: 1.7;
    }

    .workflow {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 18px;
      counter-reset: step;
    }

    .workflow-step {
      position: relative;
      padding: 22px;
      padding-top: 56px;
      border-radius: 24px;
      border: 1px solid var(--line);
      background: rgba(255, 255, 255, 0.88);
    }

    .workflow-step::before {
      counter-increment: step;
      content: "0" counter(step);
      position: absolute;
      top: 18px;
      left: 18px;
      color: var(--accent);
      font-size: 0.84rem;
      font-weight: 700;
      letter-spacing: 0.12em;
    }

    .workflow-step strong {
      display: block;
      margin-bottom: 8px;
      font-size: 1.05rem;
    }

    .workflow-step p {
      margin: 0;
      color: var(--muted);
      line-height: 1.7;
    }

    .code-panel {
      padding: 22px;
      border-radius: 22px;
      background: #0f1d38;
      color: #dce7ff;
      overflow: auto;
      box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05);
    }

    .code-panel pre {
      margin: 0;
      font-size: 0.95rem;
      line-height: 1.7;
    }

    .live-grid {
      display: grid;
      grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
      gap: 18px;
    }

    .live-card {
      padding: 24px;
      border: 1px solid var(--line);
      border-radius: 24px;
      background: rgba(255, 255, 255, 0.9);
    }

    .live-card h3 {
      margin: 0 0 8px;
      font-size: 1.2rem;
      letter-spacing: -0.03em;
    }

    .live-card > p {
      margin: 0 0 18px;
      color: var(--muted);
      line-height: 1.7;
    }

    .field-grid {
      display: grid;
      gap: 14px;
    }

    .field-grid.autocomplete-grid {
      grid-template-columns: minmax(0, 1.45fr) 210px;
    }

    .field-grid.resolve-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .card-head,
    .split-head {
      display: flex;
      align-items: start;
      justify-content: space-between;
      gap: 18px;
      margin-bottom: 18px;
    }

    .card-head > div:first-child p,
    .split-head > div:first-child p {
      margin: 0;
      color: var(--muted);
      line-height: 1.7;
    }

    .aside-note,
    .resolve-mode {
      padding: 14px 16px;
      border-radius: 18px;
      border: 1px solid var(--line);
      background: rgba(245, 249, 255, 0.92);
      color: var(--muted);
    }

    .aside-note {
      min-width: 220px;
    }

    .aside-note strong,
    .resolve-mode strong {
      display: block;
      margin-bottom: 6px;
      color: var(--ink);
      font-size: 0.95rem;
    }

    .optional-badge {
      display: inline-flex;
      align-items: center;
      padding: 6px 10px;
      border-radius: 999px;
      background: var(--accent-soft);
      color: var(--accent-strong);
      font-size: 0.75rem;
      font-weight: 700;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }

    .field input {
      width: 100%;
      border: 1px solid var(--line-strong);
      border-radius: 16px;
      background: white;
      color: var(--ink);
      padding: 15px 16px;
      font: inherit;
      outline: none;
      transition: border-color 150ms ease, box-shadow 150ms ease, transform 150ms ease;
    }

    .field input:focus {
      border-color: rgba(21, 101, 247, 0.42);
      box-shadow: 0 0 0 4px rgba(21, 101, 247, 0.1);
      transform: translateY(-1px);
    }

    .field-note {
      margin-top: 10px;
      font-size: 0.92rem;
    }

    .resolve-split {
      display: grid;
      gap: 16px;
    }

    .resolve-mode {
      display: grid;
      gap: 14px;
    }

    .resolve-divider {
      display: flex;
      align-items: center;
      gap: 14px;
      color: var(--soft);
      font-size: 0.82rem;
      font-weight: 700;
      letter-spacing: 0.14em;
      text-transform: uppercase;
    }

    .resolve-divider::before,
    .resolve-divider::after {
      content: "";
      flex: 1;
      height: 1px;
      background: var(--line);
    }

    .live-meta {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      margin: 18px 0 14px;
    }

    .meta-chip {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 10px 12px;
      border-radius: 999px;
      background: #f5f9ff;
      border: 1px solid var(--line);
      color: var(--muted);
      font-size: 0.9rem;
    }

    .meta-chip strong {
      color: var(--ink);
    }

    .live-results,
    .resolve-results {
      display: grid;
      gap: 12px;
    }

    .live-result,
    .resolved-card,
    .live-empty,
    .error {
      padding: 16px 18px;
      border-radius: 18px;
      border: 1px solid var(--line);
      background: rgba(247, 251, 255, 0.94);
    }

    .live-result {
      animation: float-in 180ms ease;
    }

    .resolved-card {
      display: grid;
      gap: 14px;
    }

    .detail-grid {
      display: grid;
      gap: 10px;
    }

    .detail-row {
      display: grid;
      grid-template-columns: 160px 1fr;
      gap: 14px;
      padding: 12px 14px;
      border-radius: 16px;
      background: white;
      border: 1px solid rgba(34, 64, 112, 0.08);
    }

    .detail-label {
      color: var(--muted);
      font-size: 0.76rem;
      font-weight: 700;
      letter-spacing: 0.1em;
      text-transform: uppercase;
    }

    .detail-value {
      word-break: break-word;
    }

    .error {
      color: #b24040;
      background: rgba(255, 236, 236, 0.95);
      border-color: rgba(178, 64, 64, 0.14);
    }

    .resolve-actions {
      display: flex;
      justify-content: flex-start;
      margin-top: 18px;
    }

    .cta-band {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 20px;
      padding: 28px 32px;
      border: 1px solid rgba(21, 101, 247, 0.16);
      border-radius: 28px;
      background: linear-gradient(135deg, #f4f9ff, #ffffff);
    }

    .cta-band strong {
      display: block;
      margin-bottom: 8px;
      font-size: 1.4rem;
      letter-spacing: -0.04em;
    }

    .cta-band p {
      margin: 0;
      color: var(--muted);
      line-height: 1.7;
    }

    .process-grid,
    .legal-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 16px;
    }

    .process-step,
    .legal-card {
      padding: 22px;
      border-radius: 22px;
      border: 1px solid var(--line);
      background: linear-gradient(180deg, rgba(255, 255, 255, 0.94), rgba(247, 251, 255, 0.9));
    }

    .process-step strong,
    .legal-card strong {
      display: block;
      margin-bottom: 8px;
      font-size: 1.02rem;
    }

    .process-step p,
    .legal-card p {
      margin: 0;
      color: var(--muted);
      line-height: 1.7;
    }

    .legal-copy {
      display: grid;
      gap: 16px;
    }

    .status-ok { color: var(--success); }
    .status-loading { color: var(--accent); }
    .status-error { color: #b24040; }

    @keyframes blink {
      0%, 49% { opacity: 1; }
      50%, 100% { opacity: 0; }
    }

    @keyframes float-in {
      from { opacity: 0; transform: translateY(10px); }
      to { opacity: 1; transform: translateY(0); }
    }

    @media (max-width: 1080px) {
      .hero,
      .live-grid,
      .cards-3,
      .workflow {
        grid-template-columns: 1fr;
      }

      .hero {
        gap: 28px;
      }
    }

    @media (max-width: 860px) {
      .field-grid.autocomplete-grid,
      .field-grid.resolve-grid,
      .cards-2,
      .process-grid,
      .legal-grid,
      .hero-metrics,
      .cta-band,
      .section-header {
        grid-template-columns: 1fr;
      }

      .section-header,
      .cta-band {
        display: grid;
      }

      .card-head,
      .split-head {
        display: grid;
      }

      .topbar {
        align-items: flex-start;
        flex-direction: column;
      }

      .hero {
        padding-top: 40px;
      }
    }

    @media (max-width: 720px) {
      main {
        width: min(var(--content), calc(100% - 18px));
        padding: 10px 0 42px;
      }

      .topbar,
      .hero,
      .section-stack {
        padding-left: 18px;
        padding-right: 18px;
      }

      .section-stack {
        padding-bottom: 18px;
      }

      .section,
      .live-card,
      .cta-band {
        padding: 20px;
      }

      .detail-row {
        grid-template-columns: 1fr;
      }

      h1 {
        max-width: none;
      }
    }
  </style>
</head>
<body>
  <main>
    <section class="page-shell">
      <header class="topbar">
        <div class="brand">
          <div class="brand-mark">AW</div>
          <div>AddressWise</div>
        </div>
        <nav class="topnav">
          <a href="#product">Product</a>
          <a href="#workflow">Workflow</a>
          <a href="#integration">Integration</a>
          <a href="#api">API</a>
          <a href="#live-demo">Live Demo</a>
          <a href="#access">Access</a>
          <a href="#about">About</a>
          <a href="#legal">Legal</a>
        </nav>
      </header>

      <section class="hero">
        <div class="hero-copy">
          <div class="eyebrow">Address infrastructure for B2B software</div>
          <h1>Make address entry feel reliable.</h1>
          <p>AddressWise is a hosted API for autocomplete and address resolution. Product teams use it to reduce form friction, improve delivery quality, and standardize messy address input before it reaches checkout, onboarding, logistics, or CRM workflows.</p>
          <div class="hero-actions">
            <a class="action action-primary" href="#live-demo">Try the live API</a>
            <a class="action action-secondary" href="#product">See how it works</a>
          </div>
          <div class="hero-metrics">
            <div class="metric">
              <strong>Autocomplete</strong>
              <span>Street-first suggestions while the user types.</span>
            </div>
            <div class="metric">
              <strong>Resolution</strong>
              <span>Free-text and structured address matching in one API.</span>
            </div>
            <div class="metric">
              <strong>Hosted</strong>
              <span>Integrated as SaaS, not sold as self-deployed software.</span>
            </div>
          </div>
        </div>

        <div class="hero-visual">
          <div class="demo-window">
            <div class="demo-window-head">
              <div class="window-dots"><span></span><span></span><span></span></div>
              <strong>Checkout Address</strong>
              <small>Live-style interaction</small>
            </div>
              <div class="typed-demo">
              <div class="product-bar">
                <div class="product-pill">Autocomplete in progress</div>
                <div id="animatedCountryBias">Country bias optional</div>
              </div>
              <div class="hero-input">
                <label>Street</label>
                <div class="typed-line"><span id="animatedQuery"></span><span class="cursor"></span></div>
              </div>
              <div class="demo-results" id="animatedResults"></div>
              <div class="demo-footnote">
                <span id="animatedStatus">Filtering candidates as the input narrows.</span>
                <span id="animatedCount">0 matches</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <div class="section-stack">
        <section class="section" id="product">
          <div class="section-header">
            <div>
              <h2>Built for high-friction address flows</h2>
              <p>Keep the product story focused: fewer failed deliveries, less manual cleanup, and cleaner customer data from the moment the user starts typing.</p>
            </div>
          </div>
          <div class="cards-3">
            <article class="card">
              <strong>Reduce drop-off</strong>
              <p>Autocomplete shortens the path to completion in checkout, onboarding, marketplace listing, and account setup flows.</p>
            </article>
            <article class="card">
              <strong>Improve downstream quality</strong>
              <p>Resolution turns messy free-text or partially structured input into a normalized address record before it reaches shipping and operations systems.</p>
            </article>
            <article class="card">
              <strong>Integrate once</strong>
              <p>One hosted service covers suggestion and resolution behavior so product, frontend, and backend teams do not need separate address handling logic.</p>
            </article>
          </div>
        </section>

        <section class="section" id="workflow">
          <div class="section-header">
            <div>
              <h2>Simple application flow</h2>
              <p>AddressWise is designed to sit inside an existing SaaS workflow, not become its own destination product.</p>
            </div>
          </div>
          <div class="workflow">
            <article class="workflow-step">
              <strong>Suggest while users type</strong>
              <p>Call the autocomplete endpoint on the street field and keep suggestions constrained to the current country or market when needed.</p>
            </article>
            <article class="workflow-step">
              <strong>Resolve on submit</strong>
              <p>Send the final free-text or structured address to the resolve endpoint and get one best candidate back with a match score.</p>
            </article>
            <article class="workflow-step">
              <strong>Store normalized data</strong>
              <p>Write the returned address record into your systems so fulfillment, support, and analytics all operate on the same shape.</p>
            </article>
          </div>
        </section>

        <section class="section" id="integration">
          <div class="section-header">
            <div>
              <h2>Easy to deploy inside customer apps</h2>
              <p>The cleanest rollout is a hosted JavaScript loader. Customers should be able to import one script, map their fields once, and let AddressWise handle autocomplete, session tracking, and resolve-on-submit behavior.</p>
            </div>
          </div>
          <div class="cards-2">
            <article class="card">
              <strong>Recommended embed shape</strong>
              <p>One script tag or module import, one configuration object, and no need for each customer team to rebuild address UX from scratch.</p>
            </article>
            <article class="card">
              <strong>Explicit integration contract</strong>
              <p>Production integrations should still understand the moving parts: which fields map to street, city, postal code, and country, how <code>session_id</code> is generated, and how a token is supplied.</p>
            </article>
          </div>
          <div class="code-panel" style="margin-top: 18px;">
            <pre>&lt;script type="module"&gt;
  import { mountAddressWise } from "https://cdn.addresswise.eu/sdk.js";

  mountAddressWise({
    token: "customer-public-token",
    form: "#checkout-address",
    fields: {
      street: "#street",
      houseNumber: "#house-number",
      city: "#city",
      postalCode: "#postal-code",
      country: "#country"
    }
  });
&lt;/script&gt;</pre>
          </div>
        </section>

        <section class="section" id="api">
          <div class="section-header">
            <div>
              <h2>Two endpoints, clear responsibility</h2>
              <p>One endpoint narrows choices during typing. One endpoint resolves the final input when quality matters.</p>
            </div>
          </div>
          <div class="cards-2">
            <article class="card">
              <strong><code>POST /autocomplete</code></strong>
              <p class="endpoint-copy">Suggestions for addresses that begin with the street, with optional country bias. Use it in forms where the user is still composing the address.</p>
            </article>
            <article class="card">
              <strong><code>POST /resolve-address</code></strong>
              <p class="endpoint-copy">Best-match address resolution for messy free text or structured fields. Use it before persisting or executing business logic.</p>
            </article>
          </div>
          <div class="code-panel" style="margin-top: 18px;">
            <pre>{
  "autocomplete": {
    "method": "POST",
    "path": "/autocomplete",
    "payload": { "query": "aven", "country_bias": "FR" }
  },
  "resolve": {
    "method": "POST",
    "path": "/resolve-address",
    "payload": { "query": "avenue de france 123 stiring wendel 57350 fr" }
  }
}</pre>
          </div>
        </section>

        <section class="section" id="live-demo">
          <div class="section-header">
            <div>
              <h2>Live API demo</h2>
              <p>Use the hosted endpoints directly from this page. The left side shows autocomplete behavior, and the right side resolves a final address candidate.</p>
            </div>
          </div>

          <div class="live-grid">
            <section class="live-card">
              <div class="card-head">
                <div>
                  <h3>Autocomplete</h3>
                  <p>Start with the street name, as a user would in a real address form.</p>
                </div>
                <div class="aside-note">
                  <strong>Country bias <span class="optional-badge">Optional</span></strong>
                  <span>Add a country only when you want suggestions nudged toward one market.</span>
                </div>
              </div>
              <div class="field-grid autocomplete-grid">
                <div class="field">
                  <label for="autocompleteQuery">Address (start with street)</label>
                  <input id="autocompleteQuery" autocomplete="off" spellcheck="false" placeholder="avenue de france..." />
                </div>
                <div class="field">
                  <label for="countryBias">Country bias</label>
                  <input id="countryBias" list="countryBiasOptions" autocomplete="off" spellcheck="false" placeholder="FR" />
                  <datalist id="countryBiasOptions"><!-- COUNTRY_BIAS_OPTIONS --></datalist>
                </div>
              </div>
              <p class="field-note">Real integrations must send a stable <code>session_id</code> across keystrokes. In production, the browser should also send a customer token or call your backend proxy.</p>
              <div class="live-meta">
                <div class="meta-chip">Normalized query <strong id="resolvedQuery">-</strong></div>
                <div class="meta-chip">Matches <strong id="matchCount">0</strong></div>
                <div class="meta-chip">Status <strong id="autocompleteStatus">idle</strong></div>
              </div>
              <div class="live-results" id="autocompleteResults">
                <div class="live-empty">Start typing to request live suggestions.</div>
              </div>
            </section>

            <section class="live-card">
              <div class="split-head">
                <div>
                  <h3>Resolve address</h3>
                  <p>Choose one input path. Use the single Query field for raw address text, or use the structured fields if your form already collects them separately.</p>
                </div>
              </div>
              <div class="resolve-split">
                <section class="resolve-mode">
                  <div>
                    <strong>Option A: Query</strong>
                    <p>Paste or type one complete address line.</p>
                  </div>
                  <div class="field">
                    <label for="resolveQuery">Query</label>
                    <input id="resolveQuery" autocomplete="off" spellcheck="false" placeholder="avenue de france 123 stiring wendel 57350 fr" />
                  </div>
                  <div class="resolve-actions">
                    <button id="resolveQueryButton" type="button" class="action action-primary">Resolve from query</button>
                  </div>
                </section>

                <div class="resolve-divider">or</div>

                <section class="resolve-mode">
                  <div>
                    <strong>Option B: Individual fields</strong>
                    <p>Use this path when your product captures address parts separately.</p>
                  </div>
                  <div class="field-grid resolve-grid">
                    <div class="field">
                      <label for="resolveCountry">Country</label>
                      <input id="resolveCountry" autocomplete="off" spellcheck="false" placeholder="FR" />
                    </div>
                    <div class="field">
                      <label for="resolveStreet">Street</label>
                      <input id="resolveStreet" autocomplete="off" spellcheck="false" placeholder="Avenue de France" />
                    </div>
                    <div class="field">
                      <label for="resolveHouseNumber">House number</label>
                      <input id="resolveHouseNumber" autocomplete="off" spellcheck="false" placeholder="123" />
                    </div>
                    <div class="field">
                      <label for="resolveCity">City</label>
                      <input id="resolveCity" autocomplete="off" spellcheck="false" placeholder="Stiring-Wendel" />
                    </div>
                    <div class="field">
                      <label for="resolvePostalCode">Postal code</label>
                      <input id="resolvePostalCode" autocomplete="off" spellcheck="false" placeholder="57350" />
                    </div>
                  </div>
                  <div class="resolve-actions">
                    <button id="resolveStructuredButton" type="button" class="action action-primary">Resolve from fields</button>
                  </div>
                </section>
              </div>
              <div class="live-meta">
                <div class="meta-chip">Normalized input <strong id="resolveResolvedQuery">-</strong></div>
                <div class="meta-chip">Score <strong id="resolveScore">-</strong></div>
                <div class="meta-chip">Status <strong id="resolveStatus">idle</strong></div>
              </div>
              <div class="resolve-results" id="resolveResults">
                <div class="live-empty">Submit an address to inspect the live result.</div>
              </div>
            </section>
          </div>
        </section>

        <section class="section" id="access">
          <div class="section-header">
            <div>
              <h2>Login and registration process</h2>
              <p>Example buyer access flow for a European B2B SaaS product. This is placeholder content and can be replaced with production policy and operations details later.</p>
            </div>
          </div>
          <div class="process-grid">
            <article class="process-step">
              <strong>1. Company registration</strong>
              <p>A prospective customer creates a workspace with company name, business email, billing country, and an administrator account. The administrator confirms that they are authorized to act for the business customer.</p>
            </article>
            <article class="process-step">
              <strong>2. Email verification and review</strong>
              <p>The administrator verifies the email address. AddressWise may delay activation until basic anti-fraud and sanctions screening is complete and may reject registrations that are incomplete, inaccurate, or high risk.</p>
            </article>
            <article class="process-step">
              <strong>3. Team access and authentication</strong>
              <p>Once approved, the administrator can invite team members, assign roles, and rotate API credentials. Each user is responsible for keeping login details confidential and for activity performed under their account.</p>
            </article>
            <article class="process-step">
              <strong>4. Suspension and termination</strong>
              <p>Access may be suspended for misuse, unpaid fees, security incidents, or suspected unlawful use. Customers may close the workspace subject to contractual notice periods, retention duties, and unpaid invoices.</p>
            </article>
          </div>
        </section>

        <section class="section" id="about">
          <div class="section-header">
            <div>
              <h2>About us</h2>
              <p>Dummy company profile for the demo page. Replace these details with the actual company record, support contacts, and operating history.</p>
            </div>
          </div>
          <div class="cards-2">
            <article class="card">
              <strong>Who we are</strong>
              <p>AddressWise Labs Europe S.à r.l. is presented here as a fictional Luxembourg-based software company focused on address quality APIs for commerce, logistics, marketplaces, and CRM platforms across the European Union.</p>
            </article>
            <article class="card">
              <strong>How we work</strong>
              <p>Our dummy team combines geodata operations, search infrastructure, and product design. We position the product as API-first infrastructure that fits into buyer journeys instead of forcing end users into a separate portal.</p>
            </article>
          </div>
        </section>

        <section class="section" id="legal">
          <div class="section-header">
            <div>
              <h2>Legal</h2>
              <p>Draft placeholder language for a hosted address service offered to EU business customers. This is product copy, not legal advice.</p>
            </div>
          </div>
          <div class="legal-copy">
            <div class="legal-grid">
              <article class="legal-card">
                <strong>Address result disclaimer</strong>
                <p>AddressWise provides probabilistic suggestions and best-match results based on the data available at the time of the request. Customers remain responsible for verifying critical addresses before shipment, service dispatch, identity checks, tax decisions, or regulatory filings. AddressWise is not liable for losses caused solely by customer reliance on an incorrect or incomplete address result where independent verification would reasonably be expected.</p>
              </article>
              <article class="legal-card">
                <strong>Service scope</strong>
                <p>The service is intended for lawful business use within professional workflows. It does not guarantee that every real-world address exists, is deliverable, or is suitable for legal service, credit decisions, or public-sector registers.</p>
              </article>
            </div>
            <div class="legal-grid">
              <article class="legal-card">
                <strong>Core terms and conditions</strong>
                <p>These demo terms assume a business-to-business contract governed by the law of an EU member state, with mandatory consumer rules excluded because the service is not offered to consumers. Customers must provide accurate account information, use the API in compliance with applicable law including GDPR, and avoid abusive traffic, reverse engineering of protected components, and unlawful enrichment of third-party datasets.</p>
              </article>
              <article class="legal-card">
                <strong>Availability, liability, and disputes</strong>
                <p>Availability commitments, support response times, data processing terms, and security measures should be set in the order form or DPA. Liability should be limited to foreseeable direct damages and capped to fees paid in an agreed reference period, except where liability cannot be limited under applicable EU law, including fraud, willful misconduct, death, or personal injury. Commercial disputes should first go through good-faith escalation before the courts or arbitral forum named in the contract.</p>
              </article>
            </div>
          </div>
        </section>

        <section class="cta-band">
          <div>
            <strong>Address entry should feel invisible.</strong>
            <p>Use the live API above to validate the product behavior your buyers will care about: fast suggestions, cleaner final data, and a smoother path through form-heavy workflows.</p>
          </div>
          <a class="action action-primary" href="#live-demo">Run the demo</a>
        </section>
      </div>
    </section>
  </main>
  <script>
    const animatedQueryEl = document.getElementById("animatedQuery");
    const animatedResultsEl = document.getElementById("animatedResults");
    const animatedStatusEl = document.getElementById("animatedStatus");
    const animatedCountEl = document.getElementById("animatedCount");
    const animatedCountryBiasEl = document.getElementById("animatedCountryBias");

    const queryEl = document.getElementById("autocompleteQuery");
    const countryEl = document.getElementById("countryBias");
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
    const resolveQueryButtonEl = document.getElementById("resolveQueryButton");
    const resolveStructuredButtonEl = document.getElementById("resolveStructuredButton");
    const resolveResolvedQueryEl = document.getElementById("resolveResolvedQuery");
    const resolveScoreEl = document.getElementById("resolveScore");
    const resolveStatusEl = document.getElementById("resolveStatus");
    const resolveResultsEl = document.getElementById("resolveResults");
    const sessionId = crypto.randomUUID();

    const animatedScenarios = [
      {
        typed: ["a", "av", "aven", "avenu", "avenue"],
        countryBias: "FR",
        status: "Showing the highest-confidence prefixes first for a France-focused checkout.",
        results: [
          { street: "Avenue de France", locality: "Stiring-Wendel", country: "FR", formatted: "Avenue de France 123, 57350 Stiring-Wendel, FR" },
          { street: "Avenue Foch", locality: "Paris", country: "FR", formatted: "Avenue Foch, 75116 Paris, FR" },
          { street: "Avenue Victor Hugo", locality: "Paris", country: "FR", formatted: "Avenue Victor Hugo, 75116 Paris, FR" }
        ]
      },
      {
        typed: ["d", "do", "dol", "dlou", "dlouh", "dlouhá"],
        countryBias: "CZ",
        status: "Country bias is optional, but it helps keep suggestions on the right market.",
        results: [
          { street: "Dlouhá", locality: "Praha", country: "CZ", formatted: "Dlouhá 731/35, 110 00 Praha 1, CZ" },
          { street: "Dlouhá", locality: "Brno", country: "CZ", formatted: "Dlouhá 12, 602 00 Brno-střed, CZ" },
          { street: "Dlouhá třída", locality: "Havířov", country: "CZ", formatted: "Dlouhá třída 464/23, 736 01 Havířov, CZ" }
        ]
      },
      {
        typed: ["fr", "fri", "frie", "fried", "friedr", "friedri", "friedric", "friedrich"],
        countryBias: "DE",
        status: "The same UI works for Germany without changing the integration contract.",
        results: [
          { street: "Friedrichstraße", locality: "Berlin", country: "DE", formatted: "Friedrichstraße 123, 10117 Berlin, DE" },
          { street: "Friedrich-Ebert-Anlage", locality: "Frankfurt am Main", country: "DE", formatted: "Friedrich-Ebert-Anlage 49, 60308 Frankfurt am Main, DE" },
          { street: "Friedrichstraße", locality: "Düsseldorf", country: "DE", formatted: "Friedrichstraße 62, 40217 Düsseldorf, DE" }
        ]
      },
      {
        typed: ["via", "via r", "via ro", "via rom", "via roma", "via roma 2", "via roma 21"],
        countryBias: "IT",
        status: "Once the user reaches a concrete address, the form can pass the final value to resolution.",
        results: [
          { street: "Via Roma", locality: "Torino", country: "IT", formatted: "Via Roma 21, 10123 Torino, IT" },
          { street: "Via Roma", locality: "Milano", country: "IT", formatted: "Via Roma 2, 20121 Milano, IT" }
        ]
      }
    ];

    let debounceTimer = null;
    let requestCounter = 0;

    queryEl.addEventListener("input", scheduleFetch);
    countryEl.addEventListener("input", scheduleFetch);

    startAnimatedDemo();

    function scheduleFetch() {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(runFetch, 140);
    }

    async function runFetch() {
      const query = queryEl.value;
      const countryBias = countryEl.value.trim();

      if (!query.trim()) {
        resolvedQueryEl.textContent = "-";
        matchCountEl.textContent = "0";
        setStatus(autocompleteStatusEl, "idle");
        autocompleteResultsEl.innerHTML = '<div class="live-empty">Start typing to request live suggestions.</div>';
        return;
      }

      const currentRequest = ++requestCounter;
      setStatus(autocompleteStatusEl, "loading");

      try {
        const response = await fetch("/autocomplete", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            session_id: sessionId,
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
        setStatus(autocompleteStatusEl, "ok");

        if (!payload.suggestions.length) {
          autocompleteResultsEl.innerHTML = '<div class="live-empty">No suggestions for this prefix.</div>';
          return;
        }

        autocompleteResultsEl.innerHTML = payload.suggestions.map((item) => `
          <article class="live-result">
            <small>${escapeHtml(item.country_code)}${item.locality ? " • " + escapeHtml(item.locality) : ""}</small>
            <strong>${escapeHtml(item.street)}</strong>
            <div class="detail-value">${escapeHtml(item.formatted)}</div>
          </article>
        `).join("");
      } catch (error) {
        if (currentRequest !== requestCounter) {
          return;
        }

        setStatus(autocompleteStatusEl, "error");
        autocompleteResultsEl.innerHTML = `<div class="error">${escapeHtml(error.message || "request failed")}</div>`;
      }
    }

    resolveQueryButtonEl.addEventListener("click", () => runResolveFetch("query"));
    resolveStructuredButtonEl.addEventListener("click", () => runResolveFetch("structured"));

    resolveQueryEl.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        event.preventDefault();
        runResolveFetch("query");
      }
    });

    [
      resolveCountryEl,
      resolveStreetEl,
      resolveHouseNumberEl,
      resolveCityEl,
      resolvePostalCodeEl
    ].forEach((el) => {
      el.addEventListener("keydown", (event) => {
        if (event.key === "Enter") {
          event.preventDefault();
          runResolveFetch("structured");
        }
      });
    });

    async function runResolveFetch(mode) {
      const payload = mode === "query"
        ? { query: emptyToNull(resolveQueryEl.value) }
        : {
            street: emptyToNull(resolveStreetEl.value),
            house_number: emptyToNull(resolveHouseNumberEl.value),
            city: emptyToNull(resolveCityEl.value),
            postal_code: emptyToNull(resolvePostalCodeEl.value),
            country: emptyToNull(resolveCountryEl.value)
          };

      if (!Object.values(payload).some(Boolean)) {
        resolveResolvedQueryEl.textContent = "-";
        resolveScoreEl.textContent = "-";
        setStatus(resolveStatusEl, "idle");
        resolveResultsEl.innerHTML = `<div class="live-empty">${mode === "query" ? "Enter a query to inspect the live result." : "Fill in at least one structured field to inspect the live result."}</div>`;
        return;
      }

      setStatus(resolveStatusEl, "loading");
      resolveResultsEl.innerHTML = '<div class="live-empty">Resolving address...</div>';

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
        setStatus(resolveStatusEl, "ok");
        resolveResultsEl.innerHTML = renderResolveResult(body);
      } catch (error) {
        setStatus(resolveStatusEl, "error");
        resolveResultsEl.innerHTML = `<div class="error">${escapeHtml(error.message || "request failed")}</div>`;
      }
    }

    function renderResolveResult(payload) {
      const address = payload.address || {};
      const diagnostics = payload.diagnostics || {};
      const rows = [
        ["Formatted", address.formatted],
        ["Country", address.country_code],
        ["Locality", address.locality],
        ["Dependent locality", address.dependent_locality],
        ["Street", address.street],
        ["House number", address.house_number],
        ["House number type", address.house_number_type],
        ["Unit", address.unit],
        ["Postal code", address.postal_code],
        ["Latitude", address.latitude],
        ["Longitude", address.longitude],
        ["Address ID", address.id],
        ["Trigram score", diagnostics.trigram_score],
        ["Levenshtein distance", diagnostics.levenshtein_distance]
      ].filter(([, value]) => value !== null && value !== undefined && String(value) !== "");

      return `
        <article class="resolved-card">
          <div>
            <small>${escapeHtml(address.country_code || "-")}${address.locality ? " • " + escapeHtml(address.locality) : ""}</small>
            <strong>${escapeHtml(address.formatted || "Resolved address")}</strong>
          </div>
          <div class="detail-grid">
            ${rows.map(([label, value]) => `
              <div class="detail-row">
                <div class="detail-label">${escapeHtml(label)}</div>
                <div class="detail-value">${escapeHtml(value)}</div>
              </div>
            `).join("")}
          </div>
        </article>
      `;
    }

    function startAnimatedDemo() {
      let scenarioIndex = 0;

      runScenario();

      async function runScenario() {
        const scenario = animatedScenarios[scenarioIndex];
        scenarioIndex = (scenarioIndex + 1) % animatedScenarios.length;
        animatedCountryBiasEl.textContent = scenario.countryBias
          ? `Country bias optional: ${scenario.countryBias}`
          : "Country bias optional";

        for (const value of scenario.typed) {
          animatedQueryEl.textContent = value;
          renderAnimatedResults(scenario.results, value);
          animatedStatusEl.textContent = scenario.status;
          animatedCountEl.textContent = `${scenario.results.length} ${scenario.results.length === 1 ? "match" : "matches"}`;
          await sleep(480);
        }

        await sleep(1500);

        for (let index = scenario.typed[scenario.typed.length - 1].length; index >= 0; index -= 1) {
          animatedQueryEl.textContent = scenario.typed[scenario.typed.length - 1].slice(0, index);
          await sleep(45);
        }

        renderAnimatedResults([], "");
        animatedStatusEl.textContent = "Filtering candidates as the input narrows.";
        animatedCountryBiasEl.textContent = "Country bias optional";
        animatedCountEl.textContent = "0 matches";
        await sleep(320);
        runScenario();
      }
    }

    function renderAnimatedResults(results, value) {
      if (!results.length || !value) {
        animatedResultsEl.innerHTML = "";
        return;
      }

      animatedResultsEl.innerHTML = results.map((item, index) => `
        <article class="demo-result${index === 0 ? " active" : ""}">
          <small>${escapeHtml(item.country)}${item.locality ? " • " + escapeHtml(item.locality) : ""}</small>
          <strong>${highlightPrefix(item.street, value)}</strong>
          <div class="detail-value">${escapeHtml(item.formatted)}</div>
        </article>
      `).join("");
    }

    function highlightPrefix(text, prefix) {
      const lowerText = text.toLowerCase();
      const lowerPrefix = prefix.toLowerCase();
      const start = lowerText.indexOf(lowerPrefix);

      if (start === -1) {
        return escapeHtml(text);
      }

      const end = start + prefix.length;
      return `${escapeHtml(text.slice(0, start))}<span style="color: var(--accent);">${escapeHtml(text.slice(start, end))}</span>${escapeHtml(text.slice(end))}`;
    }

    function setStatus(element, value) {
      element.textContent = value;
      element.className = "";
      if (value === "ok") {
        element.classList.add("status-ok");
      } else if (value === "loading") {
        element.classList.add("status-loading");
      } else if (value === "error") {
        element.classList.add("status-error");
      }
    }

    function sleep(ms) {
      return new Promise((resolve) => setTimeout(resolve, ms));
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
