// Run against a `marqdo view output` page or a live view URL.
// MARQDO_PLAYWRIGHT_MODULE may point to an existing playwright-core installation.
const assert = require('node:assert/strict');
const { chromium } = require(process.env.MARQDO_PLAYWRIGHT_MODULE || 'playwright');
(async () => {
  const browser = await chromium.launch({headless:true, executablePath:process.env.MARQDO_CHROMIUM || undefined, args:['--no-sandbox']});
  const page = await browser.newPage({viewport:{width:1440,height:1000}});
  const errors = []; page.on('pageerror',e=>errors.push(e.message));
  // Offline assets must work even when every CDN is unavailable.
  await page.route('https://**/*',route=>route.abort());
  await page.goto(process.argv[2]); await page.waitForTimeout(400);
  assert.equal(await page.locator('#structure').count(),1,'unique content target');
  for (const width of [1440,1100,1024,1000,800,390]) {
    await page.setViewportSize({width,height:1000}); await page.waitForTimeout(100);
    assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'horizontal overflow at '+width);
    const notes=await page.locator('.is-margin-note').count();
    if(width>1000) assert(notes>0,'wide view should have margin notes');
    else assert.equal(notes,0,'compact view should use inline notes');
    const rects=await page.locator('.is-margin-note').evaluateAll(nodes=>nodes.map(n=>{const r=n.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom};}));
    rects.forEach(r=>assert(r.left>=0&&r.right<=width,'offscreen note'));
    for(let i=0;i<rects.length;i++) for(let j=i+1;j<rects.length;j++) {
      const a=rects[i],b=rects[j]; assert(!(a.left<b.right&&b.left<a.right&&a.top<b.bottom&&b.top<a.bottom),'overlapping notes');
    }
  }
  await page.setViewportSize({width:1440,height:1000});
  await page.locator('[data-reader-action="functions"]').click();
  assert(await page.locator('#reader-dialog #fn-outline').count());
  await page.keyboard.press('Escape'); await page.locator('#outline-rail #fn-outline').waitFor({state:'attached'}); assert(await page.locator('#outline-rail #fn-outline').count(),'restore real function panel');
  await page.locator('[data-reader-action="variables"]').click();
  assert(await page.locator('#reader-dialog #vars-panel').count());
  await page.keyboard.press('Escape'); await page.locator('#outline-rail #vars-panel').waitFor({state:'attached'}); assert(await page.locator('#outline-rail #vars-panel').count());
  await page.locator('[data-reader-action="search"]').click();
  await page.locator('#reader-dialog input').fill('main'); assert(await page.locator('#reader-dialog .reader-option').count());
  await page.keyboard.press('Escape');
  const visibleChip=page.locator('.annot-chip:visible').first(); await visibleChip.click(); assert(await page.locator('#reader-dialog pre').isVisible()); await page.keyboard.press('Escape');
  for (const picker of await page.locator('.call-picker,.loop-picker').all()) {
    const opts=await picker.locator('option').count(); if(opts>1) {await picker.selectOption({index:0});await picker.selectOption({index:opts-1});}
  }
  await page.locator('[data-reader-action="settings"]').click();
  await page.getByRole('button',{name:'切换标准 / 大号正文'}).click(); await page.keyboard.press('Escape');
  await page.locator('[data-reader-action="memory"]').click(); assert((await page.locator('#reader-dialog pre').textContent()).includes('events')); await page.keyboard.press('Escape');
  const live=await page.evaluate(()=>!!window.MARQDO_VIEW_LIVE);
  if(live){
    const response=page.waitForResponse(r=>r.url().endsWith('/api/session/run_document'));
    await page.locator('[data-reader-action="run"]').click(); const result=await (await response).json();
    assert(result.ok&&result.structure_html&&result.trace_json,'live run must return matching annotations');
    await page.waitForTimeout(200); assert(await page.locator('.is-margin-note').count()>0,'annotations survive rerun');
  } else {
    await page.locator('[data-reader-action="run"]').click(); assert((await page.locator('.reader-toast').textContent()).includes('静态页面'));
  }
  assert.deepEqual(errors,[],'browser JavaScript errors');
  const out=process.env.MARQDO_SCREENSHOT_DIR;
  if(out){await page.screenshot({path:out+'/desktop.png'});await page.setViewportSize({width:390,height:844});await page.waitForTimeout(100);await page.screenshot({path:out+'/mobile.png'});}
  console.log(JSON.stringify({ok:true,live,sizes:6,errors,formulaCount:await page.locator('.katex').count()}));
  await browser.close();
})().catch(e=>{console.error(e);process.exit(1);});
