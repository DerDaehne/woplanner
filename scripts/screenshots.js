const { chromium } = require('playwright-core');
const fs = require('fs');

const EXE = '/nix/store/kjnpy3lhy34092p73iz82bq2bzhfb7jm-playwright-chromium/chrome-linux64/chrome';
const OUT = process.env.OUT;
const BASE = 'http://localhost:3000';

(async () => {
  const browser = await chromium.launch({ executablePath: EXE });
  const ctx = await browser.newContext({
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: 2,
    isMobile: true,
    hasTouch: true,
  });
  const page = await ctx.newPage();

  const shot = async (name, url) => {
    await page.goto(BASE + url, { waitUntil: 'networkidle' });
    await page.waitForTimeout(400);
    await page.screenshot({ path: `${OUT}/${name}.png`, fullPage: true });
    console.log(name, '->', page.url());
  };

  await shot('01-users', '/users');
  await ctx.request.post(BASE + '/users/user-demo-001/select');

  await shot('02-dashboard', '/dashboard');
  await shot('03-workouts', '/workouts');
  await shot('04-workout-detail', '/workouts/wo-push-001');
  await shot('05-exercises', '/exercises');
  await shot('06-progression', '/exercises/ex-bench-press-001/progression');
  await shot('07-history', '/history');
  await shot('08-history-detail', '/history/1e156bda-790f-49d1-81e5-494cb0ea80e8');

  // Live training: start a session, then screenshot it
  const r = await ctx.request.post(BASE + '/start-training', {
    form: { workout_id: 'wo-push-001' },
  });
  const body = await r.text();
  fs.writeFileSync(`${OUT}/start-training.txt`, r.status() + '\n' + body.slice(0, 2000));
  const m = body.match(/live-training\/([a-z0-9-]+)/i);
  if (m) {
    await shot('09-live-training', '/live-training/' + m[1]);
  } else {
    console.log('no live training id found');
  }

  await shot('10-error', '/workouts/does-not-exist');

  await browser.close();
})();
