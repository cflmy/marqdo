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
  for (const width of [1920,1440,1100,1024,1000,800,390]) {
    await page.setViewportSize({width,height:1000}); await page.waitForTimeout(100);
    assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'horizontal overflow at '+width);
    const notes=await page.locator('.is-margin-note').count();
    if(width>1000) assert(notes>0,'wide view should have margin notes');
    else assert.equal(notes,0,'compact view should use inline notes');
    const rects=await page.locator('.is-margin-note').evaluateAll(nodes=>nodes.map(n=>{const r=n.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom};}));
    rects.forEach(r=>assert(r.left>=0&&r.right<=width,'offscreen note'));
    if(width>=1440) await checkConnections();
    for(let i=0;i<rects.length;i++) for(let j=i+1;j<rects.length;j++) {
      const a=rects[i],b=rects[j]; assert(!(a.left<b.right&&b.left<a.right&&a.top<b.bottom&&b.top<a.bottom),'overlapping notes');
    }
  }
  async function checkConnections() {
    const issues = await page.evaluate(() => {
      const issues=[];
      document.querySelectorAll('.reader-connector').forEach(group => {
        const path=group.querySelector('.reader-leader');
        const anchor=Array.from(document.querySelectorAll('[data-reader-source-anchor]')).find(a=>a.dataset.readerSourceAnchor===group.dataset.readerAnchor);
        if(!anchor){issues.push('missing actual source anchor');return;}
        const chip=Array.from(document.querySelectorAll('.is-margin-note')).find(c=>c.dataset.annotId===group.dataset.annotId);
        const p=path.getPointAtLength(0), start=new DOMPoint(p.x,p.y).matrixTransform(path.getScreenCTM());
        let rects;
        if(anchor.matches('.mq-anchor')) rects=Array.from(anchor.getClientRects());
        else{const range=document.createRange();range.selectNodeContents(anchor);rects=Array.from(range.getClientRects());}
        if(!rects.some(r=>Math.abs(start.x-(chip.dataset.readerSide==='left'?r.left-2:r.right+2))<2&&Math.abs(start.y-(r.bottom+2))<2)) issues.push('connector starts away from its source');
        const label=chip.querySelector('.annot-label').getBoundingClientRect();
        const q=path.getPointAtLength(path.getTotalLength()), finish=new DOMPoint(q.x,q.y).matrixTransform(path.getScreenCTM());
        const expectedX=chip.dataset.readerSide==='left'?label.right+9:label.left-9;
        if(Math.abs(finish.x-expectedX)>2||Math.abs(finish.y-(label.top+label.height/2))>2) issues.push('connector misses the annotation label');
        if((path.getAttribute('d').match(/C /g)||[]).length<2) issues.push('missing Bézier curve');
        if(path.getTotalLength()<80) issues.push('connector is too short at desktop width');
        if(/(?:^|\s)L(?:\s|$)/.test(path.getAttribute('d'))) issues.push('straight rail segment');
        let bow=0;const length=path.getTotalLength();
        for(let i=1;i<20;i++){
          const t=path.getPointAtLength(length*i/20);
          const screen=new DOMPoint(t.x,t.y).matrixTransform(path.getScreenCTM());
          bow=Math.max(bow,Math.abs((finish.x-start.x)*(start.y-screen.y)-(start.x-screen.x)*(finish.y-start.y))/Math.hypot(finish.x-start.x,finish.y-start.y));
        }
        if(bow<6) issues.push('curve is visually flat');
      });
      const routes=Array.from(document.querySelectorAll('.reader-connector')).map(group=>{
        const path=group.querySelector('.reader-leader'),length=path.getTotalLength(),matrix=path.getScreenCTM();
        return {anchor:group.dataset.readerAnchor,points:Array.from({length:41},(_,i)=>{
          const p=path.getPointAtLength(length*i/40);return new DOMPoint(p.x,p.y).matrixTransform(matrix);
        })};
      });
      for(let i=0;i<routes.length;i++)for(let j=i+1;j<routes.length;j++){
        if(routes[i].anchor===routes[j].anchor)continue;
        const a=routes[i].points,b=routes[j].points;
        for(let p=1;p<a.length;p++)for(let q=1;q<b.length;q++){
          const u=a[p-1],v=a[p],w=b[q-1],z=b[q];
          const denominator=(v.x-u.x)*(z.y-w.y)-(v.y-u.y)*(z.x-w.x);
          if(Math.abs(denominator)<.001)continue;
          const t=((w.x-u.x)*(z.y-w.y)-(w.y-u.y)*(z.x-w.x))/denominator;
          const k=((w.x-u.x)*(v.y-u.y)-(w.y-u.y)*(v.x-u.x))/denominator;
          if(t>0&&t<1&&k>0&&k<1)issues.push('crossed execution connectors');
        }
      }
      return issues;
    });
    assert.deepEqual(issues,[],'actual-source curve geometry');
    assert.equal(await page.locator('.reader-connector').count(),await page.locator('.is-margin-note').count(),'one real connector per margin note');
  }
  await page.setViewportSize({width:1440,height:1000}); await page.waitForTimeout(100);
  await checkConnections();
  assert.deepEqual(await page.locator('.reader-dock-label').allTextContents(),['刷新','设置','运行','清理','更多']);
  // Another frame's annotation ID must resolve by source span, never by the
  // first declaration of the same kind on the line. Also exercise wrapped marks.
  await page.evaluate(()=>{
    const fixture=document.createElement('div');fixture.id='reader-source-regression';fixture.className='doc-block doc-exec';fixture.dataset.line='999';
    fixture.innerHTML='<div class="doc-line"><mark class="mq-anchor mq-decl" data-line="999" data-byte-start="0" data-byte-end="5" data-annot-id="frame-one-left">left</mark> then <mark class="mq-anchor mq-decl" data-line="999" data-byte-start="20" data-byte-end="26" data-annot-id="frame-one-right">right</mark></div><div class="annot-band" data-annot-band><div class="annot-chips"><button class="annot-chip annot-bind" data-annot-id="frame-two-right" data-annot-kind="bind" data-anchor-line="999" data-anchor-byte-start="20" data-anchor-byte-end="26" title="right=2"><span class="annot-label">right</span><span class="annot-value">2</span></button></div></div>';
    const line=document.createElement('div');line.className='doc-line';
    line.innerHTML='<mark class="mq-anchor mq-code" data-line="1000" data-byte-start="0" data-byte-end="300" data-annot-id="frame-one-wrap">'+('wrapped source token '.repeat(14))+'</mark>';
    fixture.insertBefore(line,fixture.querySelector('[data-annot-band]'));
    fixture.querySelector('.annot-chips').insertAdjacentHTML('beforeend','<button class="annot-chip annot-expr" data-annot-id="frame-two-wrap" data-annot-kind="expr" data-anchor-line="1000" data-anchor-byte-start="0" data-anchor-byte-end="300" title="wrapped=3"><span class="annot-label">wrapped</span><span class="annot-value">3</span></button>');
    document.querySelector('.document-surface').appendChild(fixture);window.MarqdoReader.refreshAnnotations();
  });
  await page.waitForTimeout(100);
  assert(await page.evaluate(()=>{
    const chip=document.querySelector('#reader-source-regression .annot-chip');
    const anchor=document.querySelector('#reader-source-regression [data-annot-id="frame-one-right"]');
    return chip.dataset.readerAnchor===anchor.dataset.readerSourceAnchor&&!!chip.dataset.readerAnchor;
  }),'second frame binds the second declaration');
  assert(await page.evaluate(()=>document.querySelector('[data-annot-id="frame-one-wrap"]').getClientRects().length>1),'wrapped source fixture');
  await checkConnections();
  await page.evaluate(()=>{document.getElementById('reader-source-regression').remove();window.MarqdoReader.refreshAnnotations();});
  await page.waitForTimeout(150);
  await page.evaluate(()=>{window.__readerOverlay=document.querySelector('.reader-connectors');});
  await page.waitForTimeout(250);
  assert(await page.evaluate(()=>window.__readerOverlay===document.querySelector('.reader-connectors')),'connector drawing must not trigger a mutation/resize loop');
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
    const opts=await picker.locator('option').count(); if(opts>1) {await picker.selectOption({index:0});await picker.selectOption({index:opts-1});await page.waitForTimeout(100);await checkConnections();}
  }
  await page.locator('[data-reader-action="settings"]').click();
  await page.getByRole('button',{name:'切换标准 / 大号正文'}).click(); await page.keyboard.press('Escape');
  await page.locator('[data-reader-action="memory"]').click(); assert((await page.locator('#reader-dialog pre').textContent()).includes('events')); await page.keyboard.press('Escape');
  const live=await page.evaluate(()=>!!window.MARQDO_VIEW_LIVE);
  if(live){
    const response=page.waitForResponse(r=>r.url().endsWith('/api/session/run_document'));
    await page.locator('[data-reader-action="run"]').click(); const result=await (await response).json();
    assert(result.ok&&result.structure_html&&result.trace_json,'live run must return matching annotations');
    await page.waitForTimeout(200); assert(await page.locator('.is-margin-note').count()>0,'annotations survive rerun');await checkConnections();
  } else {
    await page.locator('[data-reader-action="run"]').click(); assert((await page.locator('.reader-toast').textContent()).includes('静态页面'));
  }
  assert.deepEqual(errors,[],'browser JavaScript errors');
  const out=process.env.MARQDO_SCREENSHOT_DIR;
  if(out){await page.screenshot({path:out+'/desktop.png'});await page.setViewportSize({width:390,height:844});await page.waitForTimeout(100);await page.screenshot({path:out+'/mobile.png'});}
  console.log(JSON.stringify({ok:true,live,sizes:7,errors,formulaCount:await page.locator('.katex').count()}));
  await browser.close();
})().catch(e=>{console.error(e);process.exit(1);});
