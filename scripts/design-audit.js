// Misst die Design-Gesetze aus adr-002-design-rack gegen die laufende App.
// Aufruf über scripts/check-design.sh, nicht direkt — das Skript setzt
// NODE_PATH und die Chromium-Binary.
//
// Hintergrund: CSS scheitert lautlos. Eine Regel mit undefiniertem Custom
// Property wird verworfen, ein erfundener Klassenname erzeugt keinen Fehler.
// Deshalb wird im echten Browser gemessen statt im Stylesheet gelesen.
const { chromium } = require('playwright-core');

const EXE = process.env.CHROMIUM_BIN;
const BASE = process.env.BASE_URL || 'http://localhost:3000';
const USER = process.env.SEED_USER_ID || 'user-demo-001';

// Routen mit Seed-IDs. Live-Training wird übersprungen, wenn keine Session läuft.
const ROUTES = JSON.parse(process.env.ROUTES);

const laws = (vw) => {
  const bodyBg = getComputedStyle(document.body).backgroundColor;
  const all = [...document.querySelectorAll('body *')];
  const hasOwnBg = (e) => {
    const bg = getComputedStyle(e).backgroundColor;
    return bg !== 'rgba(0, 0, 0, 0)' && bg !== bodyBg;
  };

  // L2 — kein horizontaler Überlauf, Layout-Container hat seitliches Padding
  const page = document.querySelector('.wo-page');
  const overflow = all
    .filter((e) => e.getBoundingClientRect().right > vw + 1)
    .map((e) => e.tagName.toLowerCase() + '.' + String(e.className.baseVal ?? e.className).trim().split(/\s+/).slice(0, 3).join('.'));

  // L1 — keine Fläche in einer Fläche
  const surfaces = all.filter((e) => hasOwnBg(e) && e.getBoundingClientRect().height > 24);
  const nested = surfaces
    .filter((e) => surfaces.some((p) => p !== e && p.contains(e)))
    .map((e) => e.tagName.toLowerCase() + '.' + String(e.className.baseVal ?? e.className).trim().split(/\s+/).slice(0, 2).join('.'));

  // L4 — Tap-Ziele mindestens 44px hoch
  const small = [...document.querySelectorAll('a, button')]
    .filter((e) => {
      const b = e.getBoundingClientRect();
      return b.height > 0 && b.height < 44;
    })
    .map((e) => e.tagName.toLowerCase() + ':' + (e.innerText || e.getAttribute('aria-label') || '?').slice(0, 20).replace(/\s+/g, ' '));

  // L3 — genau eine h1
  const h1 = document.querySelectorAll('h1').length;

  // L6 — kein Emoji in Überschriften
  const emojiHeads = [...document.querySelectorAll('h1, h2, h3')]
    .filter((e) => /\p{Extended_Pictographic}/u.test(e.innerText))
    .map((e) => e.innerText.slice(0, 30).replace(/\s+/g, ' '));

  return {
    scrollW: document.documentElement.scrollWidth,
    vw,
    overflow: [...new Set(overflow)].slice(0, 5),
    padInline: page ? getComputedStyle(page).paddingInlineStart : null,
    nested: [...new Set(nested)].slice(0, 5),
    small: [...new Set(small)].slice(0, 5),
    h1,
    emojiHeads: emojiHeads.slice(0, 5),
  };
};

(async () => {
  const browser = await chromium.launch({ executablePath: EXE });
  const ctx = await browser.newContext({
    viewport: { width: 390, height: 844 },
    isMobile: true,
    hasTouch: true,
  });
  const page = await ctx.newPage();
  await ctx.request.post(`${BASE}/users/${USER}/select`);

  // Welche Gesetze bereits scharf geschaltet sind. Die übrigen Tickets des
  // Epics schalten ihres frei, sobald sie umgesetzt sind.
  const ACTIVE = (process.env.ACTIVE_LAWS || 'L2').split(',');
  let failures = 0;

  for (const [name, url] of Object.entries(ROUTES)) {
    await page.goto(BASE + url, { waitUntil: 'networkidle' });
    const r = await page.evaluate(laws, 390);
    const bad = [];

    if (ACTIVE.includes('L2')) {
      if (r.scrollW > r.vw) bad.push(`L2 horizontaler Überlauf: scrollWidth ${r.scrollW} > ${r.vw} [${r.overflow.join(', ')}]`);
      if (!r.padInline || parseFloat(r.padInline) <= 0) bad.push(`L2 .wo-page ohne seitliches Padding: ${r.padInline}`);
    }
    if (ACTIVE.includes('L1') && r.nested.length) bad.push(`L1 verschachtelte Flächen: ${r.nested.join(', ')}`);
    if (ACTIVE.includes('L3') && r.h1 !== 1) bad.push(`L3 ${r.h1} h1-Elemente statt genau einem`);
    if (ACTIVE.includes('L4') && r.small.length) bad.push(`L4 Tap-Ziel unter 44px: ${r.small.join(', ')}`);
    if (ACTIVE.includes('L6') && r.emojiHeads.length) bad.push(`L6 Emoji in Überschrift: ${r.emojiHeads.join(' | ')}`);

    if (bad.length) {
      failures += bad.length;
      console.log(`FAIL ${name}`);
      bad.forEach((b) => console.log(`     ${b}`));
    } else {
      console.log(`ok   ${name}  scrollW=${r.scrollW} pad=${r.padInline} h1=${r.h1} nested=${r.nested.length}`);
    }
  }

  await browser.close();
  process.exit(failures ? 1 : 0);
})();
