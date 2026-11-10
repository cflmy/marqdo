// Read typed details in the actual static/live reader without executing payloads.
const assert=require('node:assert/strict');
const {chromium}=require(process.env.MARQDO_PLAYWRIGHT_MODULE||'playwright');
let browser;
(async()=>{
  browser=await chromium.launch({headless:true,args:['--no-sandbox']});
  const context=await browser.newContext({viewport:{width:1440,height:1000}});
  await context.route('https://**/*',r=>r.request().isNavigationRequest() ? r.fulfill({contentType:'text/html',body:'<h1>navigation reached</h1>'}) : r.abort());
  const page=await context.newPage(),errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  await page.goto(process.argv[2]);await page.waitForTimeout(250);
  const label=text=>page.locator('#structure .occl-label').getByText(text,{exact:true});
  const ids=await page.locator('#structure .occl-label').evaluateAll(ns=>ns.map(n=>n.dataset.logicId));
  assert(ids.length>=10);assert.equal(new Set(ids).size,ids.length);
  for(const id of ids)assert.equal(await page.locator('#'+id+'-note').count(),1,'one detail per description');
  const callId=await label('函数与参数').getAttribute('data-logic-id');
  const callNote=page.locator('#'+callId+'-note');
  assert.equal(await callNote.getAttribute('data-logic-kind'),'call');
  assert.deepEqual(await callNote.locator('.logic-arguments dt').allTextContents(),['单价','数量']);
  const definition=await callNote.locator('.logic-definition').getAttribute('href');
  await label('函数与参数').click();
  assert(await page.locator('#reader-dialog').evaluate(n=>n.open));
  assert(await page.locator('#reader-dialog .logic-arguments').isVisible());
  assert.equal(await page.locator('#reader-dialog .logic-call-name').first().textContent(),'计算费用');
  await page.locator('#reader-dialog .logic-source > summary').click();
  assert((await page.locator('#reader-dialog pre').innerText()).includes('计算费用(单价=12, 数量=2)'));
  await page.locator('#reader-dialog .logic-definition').click();
  assert.equal(await page.locator('#reader-dialog').evaluate(n=>n.open),false);
  assert.equal(new URL(page.url()).hash,definition);
  const nestedId=await label('嵌套调用').getAttribute('data-logic-id');
  assert.equal(await page.locator('#'+nestedId+'-note .logic-call-name').count(),2);
  const listId=await label('列表内容').getAttribute('data-logic-id');
  assert.deepEqual(await page.locator('#'+listId+'-note .logic-values li').allTextContents(),['1','2','3']);
  const recordId=await label('记录内容').getAttribute('data-logic-id');
  assert((await page.locator('#'+recordId+'-note .logic-body').textContent()).includes('茶'));
  assert.equal(await page.locator('#structure .occl-stub').count(),1);
  const repository=label('访问项目仓库');
  assert.equal(await repository.evaluate(n=>n.tagName),'A');
  const repoNote=page.locator('#'+await repository.getAttribute('data-logic-id')+'-note');
  if(!await repoNote.evaluate(n=>n.open))await repoNote.locator(':scope > summary').click();
  const popupPromise=context.waitForEvent('page');await repoNote.locator('.logic-link-target').click();
  const popup=await popupPromise;await popup.waitForLoadState();
  assert.equal(popup.url(),'https://github.com/cflmy/marqdo');await popup.close();
  await label('跳到函数定义').click();assert.equal(new URL(page.url()).hash,definition);
  await label('查看阅读示例').click();await page.waitForLoadState();
  assert(page.url().includes('view-reading.mq.md'));
  assert(await page.locator('#structure').getByText('购买两件商品',{exact:true}).count());
  await page.goto(process.argv[2]);await page.waitForTimeout(200);
  for(const width of [1920,1440,1100,1000,800,390]){
    await page.setViewportSize({width,height:1000});await page.waitForTimeout(150);
    assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'overflow '+width);
    assert.deepEqual(await page.locator('.is-margin-note').evaluateAll(nodes=>{
      const issues=[],rects=nodes.map(n=>n.getBoundingClientRect());
      nodes.forEach((n,i)=>{
        const r=rects[i];if(r.left<0||r.right>innerWidth)issues.push('offscreen');
        const source=document.querySelector('[data-reader-source-anchor="'+n.dataset.readerAnchor+'"]');
        if(!source||source.dataset.logicId!==n.dataset.annotId)issues.push('wrong source');
        for(let j=i+1;j<rects.length;j++){const b=rects[j];if(r.left<b.right&&b.left<r.right&&r.top<b.bottom&&b.top<r.bottom)issues.push('overlap');}
      });
      if(innerWidth>=1440&&!nodes.length)issues.push('missing wide notes');
      if(innerWidth<=1000&&nodes.length)issues.push('mobile margin');
      return issues;
    }),[],'precise and readable notes '+width);
  }
  const mobileNote=page.locator('#'+callId+'-note');
  await mobileNote.locator(':scope > summary').click();
  assert(await mobileNote.locator('.logic-body').isVisible());
  await page.setViewportSize({width:430,height:1000});await page.waitForTimeout(150);
  assert(await mobileNote.evaluate(n=>n.open),'manual disclosure survives layout');
  if(process.env.MARQDO_SCREENSHOT_DIR){
    await page.screenshot({path:process.env.MARQDO_SCREENSHOT_DIR+'/details-mobile.png',fullPage:true});
    await page.setViewportSize({width:1440,height:1000});await page.waitForTimeout(200);
    await page.locator('#structure .doc-prose').first().scrollIntoViewIfNeeded();
    await page.screenshot({path:process.env.MARQDO_SCREENSHOT_DIR+'/details-desktop.png'});
    await label('函数与参数').click();
    await page.screenshot({path:process.env.MARQDO_SCREENSHOT_DIR+'/details-call.png'});
    await page.keyboard.press('Escape');
  }
  const plainContext=await browser.newContext({javaScriptEnabled:false,viewport:{width:390,height:900}});
  const plain=await plainContext.newPage();await plain.goto(process.argv[2]);
  const note=plain.locator('#'+callId+'-note');await note.locator(':scope > summary').click();
  assert(await note.locator('.logic-arguments').isVisible());
  await note.locator('.logic-source > summary').click();assert((await note.locator('pre').innerText()).includes('计算费用'));
  await plain.locator('#structure a.occl-label').getByText('查看阅读示例',{exact:true}).click();await plain.waitForLoadState();
  assert(plain.url().includes('view-reading.mq.md'));await plainContext.close();
  assert.deepEqual(errors,[]);await browser.close();
  console.log('typed details: calls, values, real links, precise notes, responsive and no-JS passed');
})().catch(async e=>{console.error(e);if(browser)await browser.close();process.exitCode=1;});
