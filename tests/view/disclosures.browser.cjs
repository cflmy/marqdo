// Reproduce user activation followed by real layout/observer and live DOM updates.
const assert=require('node:assert/strict');
const {chromium}=require(process.env.MARQDO_PLAYWRIGHT_MODULE||'playwright');
let browser;
(async()=>{
  browser=await chromium.launch({headless:true,args:['--no-sandbox']});
  const page=await browser.newPage({viewport:{width:1100,height:1000}});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.route('https://**/*',r=>r.abort());
  await page.goto(process.argv[2]);
  await page.waitForTimeout(300);
  for(const width of [1100,390,1440]){
    await page.setViewportSize({width,height:1000});await page.waitForTimeout(200);
    const note=page.locator('.annot-implementation:not(.is-margin-note)').first();
    assert(await note.count(),'actual inline fallback at '+width);
    const id=await note.getAttribute('data-annot-id');
    const summary=note.locator(':scope > summary');
    if(await note.evaluate(n=>n.open))await summary.click();
    await summary.click();await page.waitForTimeout(300);
    assert(await note.evaluate(n=>n.open),'click remains open after observer layout at '+width);
    await page.evaluate(()=>{for(let i=0;i<5;i++)window.MarqdoReader.refreshAnnotations();});
    await page.setViewportSize({width:width+15,height:980});await page.waitForTimeout(200);
    assert(await note.evaluate(n=>n.open),'open choice survives resize at '+width);
    const raw=note.locator('.logic-source');
    if(await raw.count()){
      if(!await raw.evaluate(n=>n.open))await raw.locator(':scope > summary').click();
      await page.waitForTimeout(200);
      assert(await note.evaluate(n=>n.open)&&await raw.evaluate(n=>n.open),'nested source stays readable');
    }
    await page.evaluate(()=>{
      const structure=document.getElementById('structure');
      const template=structure.cloneNode(true);
      template.querySelectorAll('.annot-implementation,.logic-source').forEach(n=>n.open=false);
      window.MarqdoReader.applyExecution({ok:true,page_token:window.MARQDO_VIEW_PAGE_TOKEN,structure_html:template.innerHTML});
    });
    await page.waitForTimeout(250);
    const refreshed=page.locator('[data-annot-id="'+id+'"].annot-implementation');
    assert(await refreshed.evaluate(n=>n.open),'live replacement preserves open choice');
    if(await raw.count())assert(await refreshed.locator('.logic-source').evaluate(n=>n.open),'live replacement preserves raw source');
    await refreshed.locator(':scope > summary').focus();await page.keyboard.press('Enter');
    await page.waitForTimeout(250);assert.equal(await refreshed.evaluate(n=>n.open),false,'keyboard close persists');
    await refreshed.locator(':scope > summary').focus();await page.keyboard.press('Space');
    await page.waitForTimeout(250);assert(await refreshed.evaluate(n=>n.open),'keyboard open persists');
    await refreshed.locator(':scope > summary').click();await page.waitForTimeout(150);
    await page.evaluate(()=>window.MarqdoReader.refreshAnnotations());await page.waitForTimeout(150);
    assert.equal(await refreshed.evaluate(n=>n.open),false,'explicit close persists');
  }
  assert.deepEqual(errors,[]);await browser.close();
  console.log('disclosures: click, keyboard, nested source, repeated layout, resize and live replacement passed');
})().catch(async e=>{console.error(e);if(browser)await browser.close();process.exitCode=1;});
