// Real-browser validation; development orchestration is skills/marqdo-dev/browser.mq.md.
import test from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createServer } from "node:net";
import { once } from "node:events";
import { fileURLToPath } from "node:url";
import { chromium } from "../.marqdo/browser-tests/node_modules/playwright/index.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
async function freePort() {
  const listener = createServer();
  listener.listen(0, "127.0.0.1");
  await once(listener, "listening");
  const port = listener.address().port;
  await new Promise(resolve => listener.close(resolve));
  return port;
}
async function serve(project, t, options = {}) {
  const port = await freePort();
  const child = spawn(root + "target/debug/marqdo", [
    "call", project + "/index.mq.md", "serve", "--args-json", "-",
    "--effects", options.effects || "plugin,fs.read,network.write", "--fs-root", project, "--json"
  ], {
    cwd: root,
    env: {
      ...process.env,
      MARQDO_EXT: root + "ext",
      MARQDO_LIB: root + "lib",
      MARQDO_WEB_PLUGIN: root + "plugins/web/build/libweb.so"
    },
    stdio: ["pipe", "pipe", "pipe"]
  });
  child.stdin.end(JSON.stringify({...options.args, port}));
  let logs = "";
  child.stdout.on("data", data => { logs = (logs + data).slice(-65536); });
  child.stderr.on("data", data => { logs = (logs + data).slice(-65536); });
  t.after(async () => {
    if (child.exitCode === null && child.signalCode === null) {
      child.kill("SIGTERM");
      await Promise.race([once(child, "exit"), pause(1000)]);
      if (child.exitCode === null && child.signalCode === null) {
        child.kill("SIGKILL");
        await once(child, "exit");
      }
    }
    if (options.cleanup) await options.cleanup();
  });
  const url = "http://127.0.0.1:" + port;
  const deadline = Date.now() + 15000;
  while (Date.now() < deadline) {
    assert.equal(child.exitCode, null, "Marqdo call exited: " + logs);
    try {
      const response = await fetch(url);
      if (response.ok) return { url, logs: () => logs };
    } catch (_) {}
    await pause(40);
  }
  assert.fail("Marqdo call did not listen: " + logs);
}
async function openBrowser(t) {
  const browser = await chromium.launch({ headless: true });
  t.after(() => browser.close());
  return browser;
}
async function waitText(page, selector, text) {
  await page.waitForFunction(({selector,text}) => document.querySelector(selector)?.textContent === text, {selector,text});
}
async function waitValue(page, selector, value) {
  await page.waitForFunction(({selector,value}) => document.querySelector(selector)?.value === value, {selector,value});
}
test("SPA uses current WASM for list/detail/edit and browser history", {timeout: 60000}, async t => {
  const server = await serve("examples/web-spa-app", t);
  const browser = await openBrowser(t);
  const page = await browser.newPage();
  const errors = [];
  page.on("pageerror", error => errors.push(String(error)));
  try {
    await page.goto(server.url);
    await page.waitForSelector('#items li[data-id="1"]');
    assert.equal(await page.locator("#items li").count(), 2);
    await page.locator('#items li[data-id="1"]').click();
    await waitText(page, "#title", "Detail");
    await waitText(page, "#detail-body", "Alpha");
    await page.locator("#btn-edit").click();
    await waitValue(page, "#edit-title", "Alpha");
    const title = 'A <tag> "quoted" & literal';
    await page.locator("#edit-title").fill(title);
    await page.locator("#btn-save").click();
    await waitText(page, "#detail-body", title);
    assert.equal(await page.locator('#items li[data-id="1"]').textContent(), title);
    assert.equal(await page.locator("#items tag, #items script").count(), 0);
    await page.locator("#nav-new").click();
    await page.locator("#edit-title").fill("Gamma");
    await page.locator("#btn-save").click();
    await page.waitForSelector('#items li[data-id="3"]', {state:"attached"});
    assert.equal(await page.locator("#items li").count(), 3);
    await page.goBack();
    await page.waitForFunction(() => location.pathname === "/edit" && !document.querySelector("#view-edit").hidden);
    await page.goBack();
    await page.waitForFunction(() => location.pathname === "/item" && !document.querySelector("#view-detail").hidden);
    await page.setViewportSize({width:380, height:780});
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
    assert.deepEqual(errors, []);
    await page.goto(server.url + "/item");
    await page.waitForSelector('#items li[data-id="1"]');
    assert.equal(new URL(page.url()).pathname, "/");
  } catch (error) {
    error.message += "\nBrowser: " + errors.join("\n") + "\nPage: " + await page.locator("body").innerText() + "\nServer: " + server.logs();
    throw error;
  }
});
test("two real browser tabs exchange text, empty snapshots and presence", {timeout:60000}, async t => {
  const server = await serve("examples/web-collab-doc", t);
  const browser = await openBrowser(t);
  const first = await browser.newPage();
  const second = await browser.newPage();
  const errors = [];
  for (const page of [first,second]) page.on("pageerror", error => errors.push(String(error)));
  try {
    await first.goto(server.url);
    await first.waitForFunction(() => document.querySelector("#log")?.textContent.includes("ws open"));
    await second.goto(server.url);
    await second.waitForFunction(() => document.querySelector("#log")?.textContent.includes("ws open"));
    await first.waitForFunction(() => document.querySelector("#presence")?.textContent.includes('"type":"presence"'));
    const text = 'First <tag> "text" & 中文';
    await first.locator("#doc").fill(text);
    await waitValue(second, "#doc", text);
    await second.locator("#doc").fill("the next local edit after a remote update");
    await waitValue(first, "#doc", "the next local edit after a remote update");
    await second.locator("#doc").fill("");
    await waitValue(first, "#doc", "");
    await first.setViewportSize({width:380,height:780});
    const viewport = await first.evaluate(() => ({
      width: innerWidth, scroll: document.documentElement.scrollWidth,
      overflowing: [...document.querySelectorAll("body *")].filter(el => el.getBoundingClientRect().right > innerWidth + 1).map(el => ({
        tag:el.tagName, id:el.id, width:el.getBoundingClientRect().width, text:el.textContent.slice(0,120)
      }))
    }));
    assert.equal(viewport.scroll <= viewport.width, true, JSON.stringify(viewport));
    assert.deepEqual(errors, []);
  } catch (error) {
    error.message += "\nBrowser: " + errors.join("\n") + "\nPage: " + await first.locator("body").innerText() + "\nServer: " + server.logs();
    throw error;
  }
});

async function nativeFileTask(functionName, args, effects, project) {
  const child = spawn(root + "target/debug/marqdo", [
    "call", "lib/fs.mq.md", functionName, "--args-json", "-", "--effects", effects,
    "--fs-root", project, "--require-ok", "--json"
  ], {cwd: root, stdio:["pipe","pipe","pipe"]});
  let stdout="", stderr="";
  child.stdout.on("data", chunk => { stdout += chunk; });
  child.stderr.on("data", chunk => { stderr += chunk; });
  child.stdin.end(JSON.stringify(args));
  const [code] = await once(child,"exit");
  assert.equal(code,0,"native file task failed: "+stdout+" "+stderr);
  const envelope=JSON.parse(stdout);
  assert.equal(envelope.ok,true);
  return envelope.result;
}
for (const [site, requiredCopy] of [["web-net-site", "Title is required"], ["web-net-site-zh", "标题不能为空"]]) {
test(site+" keeps native echo, current data, form retry and authenticated administration", {timeout:60000}, async t => {
  const project="examples/"+site;
  const owned=await nativeFileTask("temp_dir",{prefix:".browser-network"},"fs.write",project);
  const filename=await nativeFileTask("resolve",{path:owned+"/site.db"},"fs.read",project);
  const server=await serve(project,t,{
    args:{url:"sqlite:"+filename},
    effects:"plugin,fs.write,network.write",
    cleanup:()=>nativeFileTask("remove_tree",{path:owned},"fs.write",project)
  });
  const browser=await openBrowser(t);
  const page=await browser.newPage();
  const errors=[];
  page.on("pageerror",error=>errors.push(String(error)));
  try {
    await page.goto(server.url);
    await waitText(page,"#live-status","connected");
    const echoed='echo <tag> "quoted" & literal';
    await page.locator("#live-msg").fill(echoed);
    await page.locator("#live-send").click();
    await waitText(page,"#live-out","echo: "+echoed+"\n");
    await page.locator("#live-msg").fill("  again  ");
    await page.locator("#live-msg").press("Enter");
    await waitText(page,"#live-out","echo: "+echoed+"\necho: again\n");
    await page.locator("#live-msg").fill("   ");
    await page.locator("#live-msg").press("Enter");
    await page.waitForTimeout(120);
    assert.equal(await page.locator("#live-out").textContent(),"echo: "+echoed+"\necho: again\n");
    assert.equal(await page.locator("#live-out tag").count(),0);
    await page.goto(server.url+"/tools");
    const tools=await page.locator("body").innerText();
    assert.ok(tools.includes("http_only") && tools.includes("pic.png") && tools.includes("image/png"));
    await page.goto(server.url+"/new");
    await page.locator('form[action="/_form/article"]').evaluate(form=>{form.noValidate=true;});
    await page.locator('form[action="/_form/article"] button[type="submit"]').click();
    await page.waitForFunction(copy=>document.body.textContent.includes(copy),requiredCopy);
    assert.ok(await page.locator('input[name="_csrf"]').inputValue());
    const title='Browser article <strong>literal</strong>';
    await page.locator('input[name="title"]').fill(title);
    await page.locator('textarea[name="body"]').fill("Actual submitted body");
    await page.locator('form[action="/_form/article"] button[type="submit"]').click();
    await page.waitForFunction(title=>document.body.textContent.includes(title),title);
    assert.equal(await page.locator(".cards h2 strong").count(),0);
    await page.goto(server.url+"/admin");
    await page.locator('input[name="username"]').fill("admin");
    await page.locator('input[name="password"]').fill("secret");
    await page.locator('form[action="/admin/login"] button[type="submit"]').click();
    await page.waitForFunction(()=>document.querySelector("h1")?.textContent==="Admin");
    assert.ok((await page.locator("body").innerText()).includes(title));
    await page.goto(server.url);
    await waitText(page,"#live-status","connected");
    await page.setViewportSize({width:380,height:780});
    assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
    assert.deepEqual(errors,[]);
  } catch(error) {
    error.message += "\nBrowser: "+errors.join("\n")+"\nPage: "+await page.locator("body").innerText()+"\nServer: "+server.logs();
    throw error;
  }
});

}

for (const [site, requiredCopy] of [["web-site", "Title is required"], ["web-site-zh", "标题不能为空"]]) {
test(site+" preserves the literary theme, slug detail, session navigation and gated desk", {timeout:60000}, async t => {
 const project="examples/"+site;
 const owned=await nativeFileTask("temp_dir",{prefix:".browser-literary"},"fs.write",project);
 const filename=await nativeFileTask("resolve",{path:owned+"/site.db"},"fs.read",project);
 const server=await serve(project,t,{
  args:{url:"sqlite:"+filename},effects:"plugin,fs.write,network.write",
  cleanup:()=>nativeFileTask("remove_tree",{path:owned},"fs.write",project)
 });
 const browser=await openBrowser(t),page=await browser.newPage(),errors=[];
 page.on("pageerror",error=>errors.push(String(error)));
 // Resource declarations remain real, but this local acceptance uses font
 // fallbacks and makes no claim about external font-service availability.
 await page.route("https://fonts.googleapis.com/**",route=>route.abort());
 await page.route("https://fonts.gstatic.com/**",route=>route.abort());
 try {
  await page.goto(server.url);
  assert.equal(await page.locator(".cards article.card").count(),8);
  assert.equal(await page.locator('head meta[property="og:type"]').getAttribute("content"),"website");
  assert.equal(await page.locator('head link[rel="stylesheet"][href*="fonts.googleapis.com"]').count(),1);
  assert.equal(await page.locator('header.topnav a[href="/login"]').count(),1);
  assert.equal(await page.locator('header.topnav a[href="/admin"]').count(),0);
  const wide=await page.evaluate(()=>{
   const css=selector=>getComputedStyle(document.querySelector(selector));
   return {
    ink:css("body").backgroundColor,gold:css(".topnav a").color,
    display:css(".site-layout").display,columns:css(".site-layout").gridTemplateColumns,
    titleFont:css(".main-intro h1").fontFamily,titleSize:parseFloat(css(".main-intro h1").fontSize),
    nav:document.querySelector(".topnav").getBoundingClientRect().top,
    side:document.querySelector("aside.side").getBoundingClientRect().left,
    main:document.querySelector("section.main").getBoundingClientRect().left,
    claims:css(".main-intro ul").display,stepPanel:css(".main-intro ol").borderLeftWidth
   };
  });
  assert.equal(wide.ink,"rgb(10, 8, 9)");
  assert.equal(wide.gold,"rgb(212, 175, 55)");
  assert.equal(wide.display,"grid");
  assert.ok(wide.main>wide.side+150,JSON.stringify(wide));
  assert.ok(wide.titleFont.includes("Great Vibes") && wide.titleSize>=48,JSON.stringify(wide));
  assert.equal(wide.claims,"flex");
  assert.equal(wide.stepPanel,"2px");
  await page.locator(".cards a.card-link").first().click();
  assert.ok(new URL(page.url()).pathname.startsWith("/post/"));
  assert.equal(await page.locator(".article .article-body.md").count(),1);
  const created='Published <strong>literal</strong>';
  const summary="A short current summary",body="## Actual full Markdown body\n\nDistinct from summary.";
  await page.goto(server.url+"/new");
  await page.locator('form[action="/_form/article"]').evaluate(form=>{form.noValidate=true;});
  await page.locator('form[action="/_form/article"] button[type="submit"]').click();
  await page.waitForFunction(copy=>document.body.textContent.includes(copy),requiredCopy);
  assert.ok(await page.locator('input[name="_csrf"]').inputValue());
  await page.locator('input[name="title"]').fill(created);
  await page.locator('input[name="slug"]').fill("after-startup");
  await page.locator('textarea[name="summary"]').fill(summary);
  await page.locator('textarea[name="body"]').fill(body);
  await page.locator('form[action="/_form/article"] button[type="submit"]').click();
  await page.waitForFunction(()=>document.querySelectorAll(".cards article.card").length===9);
  assert.equal(await page.locator(".cards h2 strong").count(),0);
  assert.ok((await page.locator(".cards").innerText()).includes(summary));
  assert.ok(!(await page.locator(".cards").innerText()).includes("Actual full Markdown body"));
  await page.goto(server.url+"/post/after-startup?slug=other");
  assert.equal(await page.locator(".article-body h2").textContent(),"Actual full Markdown body");
  assert.ok(!(await page.locator(".article").innerText()).includes(summary));
  await page.goto(server.url+"/admin");
  assert.equal(new URL(page.url()).pathname,"/login");
  await page.locator('input[name="username"]').fill("admin");
  await page.locator('input[name="password"]').fill("demo");
  await page.locator('form[action="/login"] button[type="submit"]').click();
  await page.waitForURL(server.url+"/admin");
  assert.equal(await page.locator("aside.side").count(),0);
  assert.equal(await page.locator(".is-desk").count(),1);
  assert.equal(await page.locator(".cards article.card").count(),9);
  assert.equal(await page.locator('form[action="/_form/article"]').count(),1);
  await page.goto(server.url);
  assert.equal(await page.locator('header.topnav a[href="/login"]').count(),0);
  assert.equal(await page.locator('header.topnav a[href="/admin"]').count(),1);
  await page.setViewportSize({width:380,height:780});
  const narrow=await page.evaluate(()=>({
   width:innerWidth,scroll:document.documentElement.scrollWidth,
   side:document.querySelector("aside.side").getBoundingClientRect().top,
   main:document.querySelector("section.main").getBoundingClientRect().top,
   columns:getComputedStyle(document.querySelector(".site-layout")).gridTemplateColumns
  }));
  assert.ok(narrow.scroll<=narrow.width && narrow.side>narrow.main,JSON.stringify(narrow));
  await page.goto(server.url+"/admin/logout");
  assert.equal(await page.locator('header.topnav a[href="/login"]').count(),1);
  await page.goto(server.url+"/admin");
  assert.equal(new URL(page.url()).pathname,"/login");
  assert.deepEqual(errors,[]);
 } catch(error) {
  error.message+="\nBrowser: "+errors.join("\n")+"\nPage: "+await page.locator("body").innerText()+"\nServer: "+server.logs();
  throw error;
 }
});
}

test("browser effects preserve literal duplicate rows, history, storage and local transports", {timeout:60000}, async t => {
  const server = await serve("examples/browser-app", t, {effects:"plugin,fs.write,network.write"});
  const browser = await openBrowser(t);
  const context = await browser.newContext({permissions:["clipboard-read", "clipboard-write"],acceptDownloads:true});
  const page = await context.newPage();
  const errors = [];
  page.on("pageerror", error => errors.push(String(error)));
  let requests = 0;
  await page.route("https://httpbin.org/uuid", route => {
    requests++;
    return route.fulfill({status:requests === 1 ? 200 : 503,body:requests === 1 ? "controlled-success" : "controlled-failure",headers:{"access-control-allow-origin":"*"}});
  });
  try {
    await page.goto(server.url);
    await page.waitForFunction(() => !document.querySelector("#draft")?.disabled);
    const literal = 'A <tag> "quoted" & literal';
    for (let i=0;i<2;i++) {
      await page.locator("#draft").fill(literal);
      await page.locator("#add").click();
      await page.waitForFunction(n => document.querySelectorAll("#items li").length === n, i+1);
    }
    assert.deepEqual(await page.locator("#items li span").allTextContents(), [literal,literal]);
    assert.equal(await page.locator("#items tag, #items script").count(), 0);
    await page.locator('#items button[data-id="1"]').click();
    assert.equal(await page.locator("#items li").count(), 1);
    assert.equal(await page.locator("#items li span").textContent(), literal);
    await page.locator("#draft").fill("third");
    await page.locator("#add").click();
    await page.waitForSelector('#items button[data-id="3"]');
    assert.equal(await page.locator("#items li").count(), 2);
    await page.locator("#copy-log").click();
    await page.waitForFunction(expected => navigator.clipboard.readText().then(actual=>actual===expected), literal+"\nthird");
    const downloadEvent = page.waitForEvent("download");
    await page.locator("#dl-list").click();
    const download = await downloadEvent;
    assert.equal(download.suggestedFilename(),"items.txt");
    const stream = await download.createReadStream();
    const chunks = [];
    for await (const chunk of stream) chunks.push(chunk);
    assert.equal(Buffer.concat(chunks).toString(),literal+"\nthird");
    await page.locator("#start-tick").click();
    await waitText(page,"#tick","1");
    await page.locator("#stop-tick").click();
    await page.waitForTimeout(1200);
    await waitText(page,"#tick","1");
    await page.locator("#draft").fill('saved <draft> "text"');
    await page.locator("#save").click();
    await waitText(page,"#log","draft saved");
    assert.equal(await page.evaluate(()=>localStorage.getItem("mq-draft")),'saved <draft> "text"');
    await page.locator("#draft").fill("changed");
    await page.locator("#load").click();
    await waitValue(page,"#draft",'saved <draft> "text"');
    const initialHistory = await page.evaluate(()=>history.length);
    await page.locator("#nav-about").click();
    await page.waitForFunction(()=>location.pathname==="/about" && !document.querySelector("#panel-about").hidden);
    await page.locator("#nav-net").click();
    await page.waitForFunction(()=>location.pathname==="/net" && !document.querySelector("#panel-net").hidden);
    assert.equal(await page.evaluate(()=>history.length),initialHistory+2);
    await page.goBack();
    await page.waitForFunction(()=>location.pathname==="/about" && !document.querySelector("#panel-about").hidden);
    await page.goBack();
    await page.waitForFunction(()=>location.pathname==="/" && !document.querySelector("#panel-home").hidden);
    await page.goForward();
    await page.goForward();
    await page.waitForFunction(()=>location.pathname==="/net" && !document.querySelector("#panel-net").hidden);
    assert.equal(await page.evaluate(()=>history.length),initialHistory+2);
    await page.locator("#fetch-all").click();
    await page.waitForFunction(()=> {
      const text=document.querySelector("#net-out").textContent;
      return text.includes("controlled-success") && text.includes("503") && text.includes("controlled-failure");
    });
    await page.locator("#ws-open").click();
    await waitText(page,"#net-out","ws open");
    const message='local <echo> "message"';
    await page.locator("#ws-msg").fill(message);
    await page.locator("#ws-send").click();
    await waitText(page,"#net-out","ws ← "+message);
    await page.locator("#ws-close").click();
    await waitText(page,"#net-out","ws closed");
    const styles = await page.evaluate(()=>({body:getComputedStyle(document.body).color,button:getComputedStyle(document.querySelector("#fetch-all")).backgroundColor}));
    assert.equal(styles.body,"rgb(26, 31, 28)");
    assert.equal(styles.button,"rgb(47, 111, 78)");
    await page.setViewportSize({width:380,height:780});
    assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
    await page.goto(server.url+"/about");
    await page.waitForFunction(()=>!document.querySelector("#draft")?.disabled);
    assert.equal(new URL(page.url()).pathname,"/");
    await waitValue(page,"#draft",'saved <draft> "text"');
    assert.equal(await page.locator("#items li").count(),0);
    assert.deepEqual(errors,[]);
  } catch(error) {
    error.message+="\nBrowser: "+errors.join("\n")+"\nPage: "+await page.locator("body").innerText()+"\nServer: "+server.logs();
    throw error;
  }
});
