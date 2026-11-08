// Exercise actual reader DOM, side-note geometry, accessibility and offline export.
const assert = require('node:assert/strict');
const { chromium } = require(process.env.MARQDO_PLAYWRIGHT_MODULE || 'playwright');
let browser;
(async()=>{
  browser=await chromium.launch({headless:true,executablePath:process.env.MARQDO_CHROMIUM||undefined,args:['--no-sandbox']});
  const page=await browser.newPage({viewport:{width:1440,height:1000}}), errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  await page.route('https://**/*',route=>route.abort());
  await page.goto(process.argv[2]);await page.waitForTimeout(300);
  const surface=page.locator('#structure');
  assert.deepEqual(await surface.locator('.occl-label').allTextContents(),['按件数计算金额','由作者补充具体业务','购买两件商品','使用默认件数','依次累计数量','输出累计值','说明输入为空']);
  assert.equal(await surface.locator('.occl-stub').count(),1);
  assert.match(await surface.locator('.occl-stub').textContent(),/待补充实际逻辑/);
  const sentence=surface.locator('.doc-prose-line').filter({hasText:'先第一笔'});
  const text=await sentence.innerText();
  assert(text.includes('购买两件商品')&&text.includes('使用默认件数'));
  assert(!text.includes('计算小计(')&&!text.includes('单价=12'));
  const actions=await surface.locator('.occl-label').evaluateAll(nodes=>nodes.map(n=>({id:n.dataset.logicId,label:n.textContent})));
  for(const a of actions){
    assert.equal(await page.locator('#'+a.id+'-note').count(),1,'one note for '+a.label);
    assert.equal(await page.locator('#'+a.id+'-source').count(),1,'unique source '+a.label);
  }
  for(const width of [1920,1440,1100,1024,1000,800,390]){
    await page.setViewportSize({width,height:1000});await page.waitForTimeout(120);
    assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'horizontal overflow '+width);
    const notes=await surface.locator('.annot-implementation.is-margin-note').count();
    if(width>=1440) assert(notes>0,'wide logic notes');
    if(width<=1000) assert.equal(notes,0,'mobile must use native disclosures');
    assert.deepEqual(await page.locator('.is-margin-note').evaluateAll(nodes=>{
      const rects=nodes.map(n=>n.getBoundingClientRect()),issues=[];
      for(let i=0;i<rects.length;i++){
        const a=rects[i];if(a.left<0||a.right>innerWidth)issues.push('offscreen');
        for(let j=i+1;j<rects.length;j++){const b=rects[j];if(a.left<b.right&&b.left<a.right&&a.top<b.bottom&&b.top<a.bottom)issues.push('overlap');}
      }return issues;
    }),[],'note geometry '+width);
  }
  await page.setViewportSize({width:1440,height:1000});await page.waitForTimeout(100);
  const source=surface.locator('.occl-label').filter({hasText:'购买两件商品'});
  await source.hover();
  const noteId=await source.getAttribute('data-logic-id');
  assert(await page.locator('#'+noteId+'-note').evaluate(n=>n.classList.contains('is-focus')),'source highlights actual implementation');
  await source.click();
  assert(await page.locator('#reader-dialog').evaluate(n=>n.open));
  assert((await page.locator('#reader-dialog-content pre').textContent()).includes('计算小计(单价=12, 数量=2)'));
  await page.keyboard.press('Escape');
  await surface.locator('.knowledge-explain').first().click();
  assert(await page.locator('.knowledge-block').evaluate(n=>n.open));
  assert((await page.locator('.knowledge-card[open]').first().innerText()).includes('每件商品的价格'));
  const technical=page.locator('.knowledge-card[open] .knowledge-technical').first();
  assert.equal(await technical.evaluate(n=>n.open),false,'identities folded initially');
  for(const href of await page.locator('.knowledge-step a,.knowledge-source-link,.knowledge-explain').evaluateAll(nodes=>nodes.map(n=>n.getAttribute('href')))){
    assert(await page.evaluate(href=>!!document.getElementById(href.slice(1)),href),'broken source link '+href);
  }
  assert.equal(await page.locator('.knowledge-error').count(),0,'knowledge must load for one example');
  await page.locator('[data-reader-action="more"]').click();
  await page.getByRole('button',{name:'可执行知识',exact:true}).click();
  assert.equal(await page.locator('#reader-dialog-content .knowledge-panel').count(),1);
  await page.keyboard.press('Escape');
  await page.waitForFunction(()=>!!document.querySelector('.knowledge-block .knowledge-panel'));
  assert.equal(await page.locator('.knowledge-block .knowledge-panel').count(),1,'restore actual panel');
  await page.locator('#reader-syntax-guide summary').click();
  assert.equal(await page.locator('[data-construct-id]').count(),19);
  // Read-only no-script fallback still exposes native details and real source text.
  const context=await browser.newContext({javaScriptEnabled:false,viewport:{width:390,height:900}});
  const plain=await context.newPage();await plain.goto(process.argv[2]);
  const note=plain.locator('.annot-implementation').first();await note.locator('summary').click();
  assert(await note.locator('.annot-value').isVisible());
  assert.equal(await plain.locator('.occl-stub').count(),1);
  await context.close();
  if(process.env.MARQDO_SCREENSHOT_DIR){
    const fs=require('node:fs');fs.mkdirSync(process.env.MARQDO_SCREENSHOT_DIR,{recursive:true});
    await page.locator('#reader-syntax-guide').evaluate(n=>n.open=false);
    await page.screenshot({path:process.env.MARQDO_SCREENSHOT_DIR+'/reading-desktop.png',fullPage:true});
    await page.setViewportSize({width:390,height:900});await page.screenshot({path:process.env.MARQDO_SCREENSHOT_DIR+'/reading-mobile.png',fullPage:true});
  }
  assert.deepEqual(errors,[]);
  await browser.close();console.log('reader readability: actions, logic notes, knowledge, source links, responsive and no-JS passed');
})().catch(async error=>{console.error(error);if(browser)await browser.close();process.exitCode=1;});
