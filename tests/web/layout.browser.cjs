// Exercise the actual recursive reference output; no external resources required.
const assert = require('node:assert/strict');
const {pathToFileURL} = require('node:url');
const {chromium} = require(process.env.MARQDO_PLAYWRIGHT_MODULE || 'playwright');
(async () => {
  const browser = await chromium.launch({headless:true, executablePath:process.env.MARQDO_CHROMIUM || undefined, args:['--no-sandbox']});
  try {
    const page = await browser.newPage();
    const errors=[]; page.on('pageerror',e=>errors.push(e.message));
    const target=process.argv[2];
    await page.goto(target.startsWith('/') ? pathToFileURL(target).href : target);
    const box=selector=>page.locator(selector).boundingBox();
    for (const width of [1440,390]) {
      await page.setViewportSize({width,height:1000});
      const brand=await box('.slot-footer-brand');
      const links=await box('.slot-footer-links');
      const docs=await box('.slot-footer-links-docs');
      const community=await box('.slot-footer-links-community');
      assert(brand&&links&&docs&&community,'all footer leaves are present');
      assert.equal(await page.locator('.slot-footer.mq-row > .slot-footer-links.mq-row').count(),1,'nested columns remain children');
      if(width===1440) {
        assert(links.x>brand.x+brand.width-1 && Math.abs(links.y-brand.y)<2,'brand and links form two columns');
        assert(community.x>docs.x+docs.width-1 && Math.abs(community.y-docs.y)<2,'links decomposes into two further columns');
      } else {
        assert(links.y>=brand.y+brand.height,'footer collapses vertically');
        assert(community.y>=docs.y+docs.height,'inner columns also collapse');
      }
      assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'no horizontal overflow');
      console.log(`layout ${width}px: outer and nested geometry passed`);
    }
    assert.deepEqual(errors,[]);
    if(process.argv[3]) await page.screenshot({path:process.argv[3],fullPage:true});
  } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exit(1);});
