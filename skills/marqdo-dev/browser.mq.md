---
markup: occlusion
title: Real browser preparation and verification through Marqdo
import:
  fs: lib/fs.mq.md
  sys: lib/sys.mq.md
  re: lib/re.mq.md
  table: lib/table.mq.md
  result: lib/result.mq.md
  engineering: ext/engineering/index.mq.md
---

Build current browser assets through the development entry's build_browser task. This module prepares the external browser driver and runs the actual UI cases, including the native browser-effects demonstration and both complete literary sites. Files and orchestration remain native Marqdo; browser automation is a documented external capability, and no Python program performs the development workflow.

## prepare

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| version | text | "1.65.0" | Pinned Playwright package version verified by this repository |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result of workspace-local dependency and Chromium preparation |

Use `version`="1.65.0" to prepare the external browser-test driver under .marqdo/browser-tests. Node, npm and Playwright are explicit test-tool boundaries; the application itself remains Marqdo, and each test starts the real server through marqdo call. Native filesystem operations create the selected workspace directories. The package and browser caches stay there, with no global npm installation or Python runner. Grant plugin, process.exec and fs.write.

Preparation is separate from verification because installing a browser does not test a page. Each command must succeed before the next one starts. The pinned version and child-only PLAYWRIGHT_BROWSERS_PATH make the driver reproducible. If a required system library is absent, preserve the actual error rather than reporting browser validation as complete. This task changes no application source or repository package manifest.

1. not [A pinned stable package version](re.is_match(text=`version`, pattern="^[0-9]+[.][0-9]+[.][0-9]+$"))
  Return *[Require an explicit stable package version](result.error(code="browser.version", message="use a pinned numeric Playwright version"))*.
We **[Create the task-owned workspace tool directory](fs.make_dirs(path=".marqdo/browser-tests"))**.
We **package = "playwright@" + version**.

`install_args` =

| argument |
|---|
| npm |
| install |
| "--prefix" |
| .marqdo/browser-tests |
| "--cache" |
| .marqdo/browser-tests/npm-cache |
| "--no-save" |
| "--package-lock=false" |
| `package` |

We **installed = [Prepare the pinned external browser driver](engineering.checked(argv=`install_args`, timeout_ms=180000, max_output=65536))**.
1. not access(`installed`, "ok")
  Return *installed*.
We **cache = sys.cwd() + "/.marqdo/browser-tests/browsers"** and **environment = [Keep browser downloads inside this workspace](table.put(in=None, at="PLAYWRIGHT_BROWSERS_PATH", value=`cache`))**.

`browser_args` =

| argument |
|---|
| node |
| .marqdo/browser-tests/node_modules/playwright/cli.js |
| install |
| chromium |

Return *[Prepare the actual headless Chromium runtime](engineering.checked(argv=`browser_args`, env=`environment`, timeout_ms=300000, max_output=65536))*.

## verify

| 参数 | 类型 | 默认 | 说明 |
|---|---|---|---|
| out | text | ".marqdo/browser-handbook-assets" | Current build_browser output directory |

| 返回 | 类型 | 说明 |
|---|---|---|
| result | map | Result from real browser effects, SPA, collaboration, bilingual network and bilingual complete-site browser cases |

Use `out`=".marqdo/browser-handbook-assets" after build_browser has produced the current WASM and prepare has installed the pinned browser driver. This task copies only the two named generated runtime assets with native Marqdo file operations; client sources remain in their project directories. Both source artifacts must exist before any example is staged. Grant plugin, process.exec and fs.write.

The external Node test harness supplies browser automation and process supervision only. It starts each application's callable serve function on a selected loopback port, then uses the actual WASM, DOM events, history and WebSocket transport. The browser-effects case verifies duplicate-row identity, literal labels, clipboard/download payloads, completed draft storage, timer cancellation, real history, ordered HTTP success/failure records and local WebSocket echo. SPA checks cover native list records, editing quoted text, new ids and back/forward behavior. Two tabs verify presence, remote snapshots, the first local edit after a remote update and an empty document. The English and Chinese network-site cases uses an owned database created and removed through native Marqdo file calls, verifying real echo, form-validation retry, current article data and authenticated administration. The complete English and Chinese sites also verify original theme declarations, current card/detail separation, real session navigation, the gated custom desk and logout. Their font requests are explicitly aborted so local acceptance proves declared families and fallback rendering, not external font delivery. Narrow viewport checks expose layout overflow. A saved native report retains actual failures; a source contract or constructed effect map is not substituted for a browser result.

We **wasm = out + "/marqdo_wasm.wasm"** and **bridge = out + "/marqdo-bridge.js"**.
1. not [The current WASM exists](fs.exists(path=`wasm`)) or not [The canonical bridge exists](fs.exists(path=`bridge`))
  Return *[Require a real current browser build](result.error(code="browser.assets", message="call build_browser before browser verification"))*.

`sites` =

| path |
|---|
| examples/browser-app/static |
| examples/web-spa-app/static |
| examples/web-collab-doc/static |
| examples/web-net-site/public/generated |
| examples/web-net-site-zh/public/generated |

- [Stage only the two generated browser resources](for sites as directory)
  We **[Keep the static destination inside the granted root](fs.make_dirs(path=`directory`))**.
  We **wasm_dest = directory + "/marqdo_wasm.wasm"** and **bridge_dest = directory + "/marqdo-bridge.js"**.
  We **[Copy the actual current WASM natively](fs.copy_file(src=`wasm`, dest=`wasm_dest`))** and **[Copy the canonical bridge natively](fs.copy_file(src=`bridge`, dest=`bridge_dest`))**.
We **cache = sys.cwd() + "/.marqdo/browser-tests/browsers"** and **environment = [Use only the prepared workspace browser cache](table.put(in=None, at="PLAYWRIGHT_BROWSERS_PATH", value=`cache`))**.

`argv` =

| argument |
|---|
| node |
| "--test" |
| tests/browser_handbook.mjs |

We **observed = [Run actual browser interaction tests](engineering.checked(argv=`argv`, env=`environment`, timeout_ms=150000, max_output=65536))**.
We **[Retain the actual test report with native IO](fs.write_json(path=".marqdo/browser-tests/result.json", value=`observed`))**.
Return *observed*.
