// All the maths runs here, so every setting updates instantly. The server
// only supplies the gem pool and prices (/api/data).
"use strict";

const COLOURS = { r: "Red", g: "Green", b: "Blue" };
const $ = (id) => document.getElementById(id);
const S = { data: null, open: "", inputOverride: {} };

// ---- settings live in the URL hash so a link reproduces the view ----
const FIELDS = ["league", "variant", "minChaos", "minListings", "crafts", "labCost"];
function readHash() {
  const h = new URLSearchParams(location.hash.slice(1));
  for (const f of FIELDS) if (h.has(f)) $(f).value = h.get(f);
  S.open = h.get("open") || "";
}
function writeHash() {
  const h = new URLSearchParams();
  for (const f of FIELDS) if ($(f).value !== "") h.set(f, $(f).value);
  if (S.open) h.set("open", S.open);
  history.replaceState(null, "", "#" + h);
}

// ---- formatting ----
function fmt(c) {
  if (c == null) return "–";
  const div = S.data && S.data.divine;
  if (div && Math.abs(c) >= div) return (c / div).toFixed(2) + " div";
  if (Math.abs(c) >= 10) return Math.round(c) + "c";
  return (Math.round(c * 10) / 10) + "c";
}
const pct = (p) => p >= 0.1 ? (p * 100).toFixed(1) + "%" : p > 0 ? (p * 100).toFixed(2) + "%" : "0%";
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
const trade = (name) => "https://www.pathofexile.com/trade/search/" + encodeURIComponent(S.data.league) +
  "?q=" + encodeURIComponent(JSON.stringify({ query: { type: name, status: { option: "securable" } }, sort: { price: "asc" } }));

// C(n, k) for small k
const C = (n, k) => (n < k ? 0 : k === 2 ? n * (n - 1) / 2 : n * (n - 1) * (n - 2) / 6);

function settings() {
  return {
    v: $("variant").value,
    minChaos: +$("minChaos").value || 0,
    minListings: +$("minListings").value || 0,
    crafts: Math.max(1, +$("crafts").value || 1),
    labCost: $("labCost").value === "" ? null : +$("labCost").value,
  };
}

function quote(g, v) {
  const q = g.q && g.q[v];
  return q ? { chaos: q[0], listings: q[1] } : null;
}

// ---- the 3-choice Font roll ----
// Sorted best-first, gem r is the pick exactly when the other two offered
// gems both rank below it: P(r) = C(n-r-1, 2) / C(n, 3).
function fontRoll(colour, st) {
  const pool = S.data.gems.filter((g) => g.font && g.colour === colour).map((g) => {
    const q = quote(g, st.v);
    const ok = q && q.chaos >= st.minChaos && q.listings >= st.minListings;
    return { g, q, value: ok ? q.chaos : 0, why: !q ? "no price" : !ok ? (q.chaos < st.minChaos ? "under min" : "few listings") : "" };
  });
  pool.sort((a, b) => b.value - a.value || ((b.q ? b.q.chaos : 0) - (a.q ? a.q.chaos : 0)));
  const n = pool.length, total = C(n, 3);
  let ev = 0;
  pool.forEach((o, r) => {
    o.p = total ? C(n - r - 1, 2) / total : 0;
    o.contrib = o.value * o.p;
    ev += o.contrib;
  });
  const zero = pool.filter((o) => o.value === 0).length;
  const hit = total ? 1 - C(zero, 3) / total : 0;

  // Cheapest gem of this colour you could put in, at this level/quality.
  let input = null;
  for (const g of S.data.gems) {
    if (!g.input || g.colour !== colour) continue;
    const q = quote(g, st.v);
    if (q && q.listings >= Math.max(st.minListings, 1) && (!input || q.chaos < input.chaos)) input = { name: g.name, chaos: q.chaos };
  }
  const override = S.inputOverride[colour];
  const inputCost = override != null ? override : input ? input.chaos : 0;
  return { colour, pool, n, ev, hit, zero, input, inputCost, net: ev - inputCost };
}

function labCost(st) {
  if (st.labCost != null) return st.labCost;
  const o = S.data.offerings || {};
  return o["Offering to the Goddess"] || 0;
}

function renderFont(st) {
  const rolls = Object.keys(COLOURS).map((c) => fontRoll(c, st));
  const best = rolls.reduce((a, b) => (b.net > a.net ? b : a));
  $("cards").innerHTML = rolls.map((r) => {
    const top = r.pool[0];
    return `<div class="card clickable ${r.colour} ${r === best ? "best" : ""} ${S.open === r.colour ? "open" : ""}" data-colour="${r.colour}">
      <span class="title">${COLOURS[r.colour]}${r === best ? " · best" : ""}</span>
      <span class="big">${fmt(r.net)}<span class="muted small"> net per craft</span></span>
      <span>Expected result <b>${fmt(r.ev)}</b></span>
      <span>Gem to put in ${r.input || S.inputOverride[r.colour] != null
        ? `<b>${fmt(r.inputCost)}</b> <span class="muted small">${S.inputOverride[r.colour] != null ? "(your price)" : esc(r.input.name)}</span>`
        : `<span class="muted">no price</span>`}
        <input class="override" data-colour="${r.colour}" type="number" min="0" step="0.5" placeholder="set" value="${S.inputOverride[r.colour] ?? ""}" title="Override the input gem price"></span>
      <span>Chance of a result worth keeping <b>${pct(r.hit)}</b></span>
      <span class="muted small">${r.n} gems in pool · ${r.zero} count as 0${top && top.value ? ` · best: ${esc(top.g.name)} ${fmt(top.value)}` : ""}</span>
    </div>`;
  }).join("");

  const lc = labCost(st);
  const run = st.crafts * best.net - lc;
  $("run").innerHTML = `Per lab run: ${st.crafts} × ${fmt(best.net)} (${COLOURS[best.colour]}) − ${fmt(lc)} lab cost =
    <b class="${run >= 0 ? "up" : "down"}">${fmt(run)}</b>`;

  const open = rolls.find((r) => r.colour === S.open);
  $("pool").innerHTML = open ? poolTable(open) : "";
}

function poolTable(r) {
  let cum = 0;
  const rows = r.pool.map((o, i) => {
    cum += o.p;
    return `<tr class="${o.value ? "" : "dim"}">
      <td class="num">${i + 1}</td>
      <td><a href="${trade(o.g.name)}" target="_blank" rel="noopener">${esc(o.g.name)}</a></td>
      <td class="num">${o.q ? fmt(o.q.chaos) : "–"}</td>
      <td class="num">${o.q ? o.q.listings : "–"}</td>
      <td class="num">${pct(o.p)}</td>
      <td class="num">${pct(cum)}</td>
      <td class="num">${o.contrib ? fmt(o.contrib) : "–"}</td>
      <td class="muted small">${o.why}</td></tr>`;
  }).join("");
  return `<h3>${COLOURS[r.colour]} pool: every possible outcome</h3>
    <p class="muted small">"Kept" is the chance this gem is the best of the three you are offered; "or better" adds up the rows above it.</p>
    <table class="grid"><thead><tr><th class="num">#</th><th>Gem</th><th class="num">Price</th><th class="num">Listings</th>
    <th class="num">Kept</th><th class="num">This or better</th><th class="num">Adds to EV</th><th></th></tr></thead><tbody>${rows}</tbody></table>`;
}

// ---- the "own transfigured version" option ----
function ownRows(st) {
  const byName = new Map(S.data.gems.map((g) => [g.name, g]));
  const fam = new Map();
  for (const g of S.data.gems) if (g.font) (fam.get(g.base) || fam.set(g.base, []).get(g.base)).push(g);
  const rows = [];
  for (const [base, forms] of fam) {
    const bq = byName.has(base) ? quote(byName.get(base), st.v) : null;
    let best = null;
    for (const f of forms) {
      const q = quote(f, st.v);
      if (q && q.listings >= st.minListings && (!best || q.chaos > best.q.chaos)) best = { g: f, q };
    }
    if (!best) continue;
    const baseCost = bq ? bq.chaos : 0;
    rows.push({ base, colour: best.g.colour, bq, best, forms: forms.length, spread: best.q.chaos - baseCost });
  }
  rows.sort((a, b) => b.spread - a.spread);
  return rows;
}

function renderOwn(st) {
  const rows = ownRows(st);
  $("bring").innerHTML = rows.slice(0, 3).map((r, i) => `<div class="card ${r.colour}">
      <span class="title">#${i + 1} bring ${esc(r.base)}</span>
      <span class="big">+${fmt(r.spread)}</span>
      <span>→ <a href="${trade(r.best.g.name)}" target="_blank" rel="noopener">${esc(r.best.g.name)}</a> ${fmt(r.best.q.chaos)}</span>
      <span class="muted small">base ${r.bq ? fmt(r.bq.chaos) : "no price"} · ${r.best.q.listings} listed</span></div>`).join("");
  const col = $("ownColour").value, find = $("ownFind").value.trim().toLowerCase();
  const shown = rows.filter((r) => (!col || r.colour === col) &&
    (!find || r.base.toLowerCase().includes(find) || r.best.g.name.toLowerCase().includes(find)));
  $("ownTable").innerHTML = `<thead><tr><th>Base gem</th><th class="num">Base price</th><th>Best form</th><th class="num">Form price</th>
    <th class="num">Listings</th><th class="num">Spread</th><th class="num">Forms</th></tr></thead><tbody>` +
    shown.map((r) => `<tr><td><span class="dot ${r.colour}"></span>${esc(r.base)}</td>
      <td class="num">${r.bq ? fmt(r.bq.chaos) : "–"}</td>
      <td><a href="${trade(r.best.g.name)}" target="_blank" rel="noopener">${esc(r.best.g.name)}</a></td>
      <td class="num">${fmt(r.best.q.chaos)}</td><td class="num">${r.best.q.listings}</td>
      <td class="num ${r.spread >= 0 ? "up" : "down"}">${fmt(r.spread)}</td><td class="num">${r.forms}</td></tr>`).join("") + "</tbody>";
}

function renderChanges() {
  const d = S.data, font = d.gems.filter((g) => g.font);
  const count = (c) => font.filter((g) => g.colour === c).length;
  $("catinfo").textContent = `${font.length} transfigured gems can come out of the Font (red ${count("r")}, green ${count("g")}, blue ${count("b")}). ` +
    `The list is rebuilt daily from GGG's own data, so a new season's gems show up by themselves; last rebuilt ${new Date(d.catalogue).toLocaleString()}.` +
    (d.uncoloured ? ` ${d.uncoloured} gems have no known colour yet and are left out.` : "");
  $("changes").innerHTML = (d.changes || []).length
    ? "<h3>Changes seen</h3>" + d.changes.map((c) => `<p><b>${new Date(c.at).toLocaleDateString()}</b>
        ${c.added && c.added.length ? `added: ${c.added.map(esc).join(", ")}` : ""}
        ${c.removed && c.removed.length ? `<br>removed: ${c.removed.map(esc).join(", ")}` : ""}</p>`).join("")
    : `<p class="muted small">No changes to the pool since this server started tracking it.</p>`;
}

function render() {
  if (!S.data) return;
  writeHash();
  const st = settings();
  const lc = S.data.offerings && S.data.offerings["Offering to the Goddess"];
  $("labCost").placeholder = lc ? "auto: " + fmt(lc) : "auto";
  renderFont(st);
  renderOwn(st);
  renderChanges();
  $("status").textContent = `${S.data.league} · prices from poe.ninja, ${new Date(S.data.prices_at).toLocaleTimeString()}` +
    (S.data.divine ? ` · 1 div = ${Math.round(S.data.divine)}c` : "");
  for (const id of ["font", "own", "pool-changes"]) $(id).hidden = false;
}

async function load() {
  $("status").textContent = "Loading prices…";
  try {
    const r = await fetch("api/data?league=" + encodeURIComponent($("league").value));
    if (!r.ok) throw new Error(await r.text());
    S.data = await r.json();
    render();
  } catch (e) {
    $("status").textContent = "Could not load: " + e.message;
  }
}

async function init() {
  const r = await fetch("api/leagues").then((r) => r.json()).catch(() => ({ leagues: ["Standard"], default: "Standard" }));
  $("league").innerHTML = r.leagues.map((l) => `<option>${esc(l)}</option>`).join("");
  $("league").value = r.default;
  $("controls").addEventListener("submit", (e) => e.preventDefault());
  readHash();
  if (![...$("league").options].some((o) => o.value === $("league").value)) $("league").value = r.default;

  $("league").addEventListener("change", load);
  for (const id of ["variant", "minChaos", "minListings", "crafts", "labCost", "ownColour", "ownFind"]) $(id).addEventListener("input", render);
  $("cards").addEventListener("click", (e) => {
    if (e.target.classList.contains("override")) return;
    const card = e.target.closest(".card");
    if (!card) return;
    S.open = S.open === card.dataset.colour ? "" : card.dataset.colour;
    render();
  });
  $("cards").addEventListener("change", (e) => {
    if (!e.target.classList.contains("override")) return;
    const v = e.target.value;
    if (v === "") delete S.inputOverride[e.target.dataset.colour];
    else S.inputOverride[e.target.dataset.colour] = +v;
    render();
  });
  load();
}

init();
