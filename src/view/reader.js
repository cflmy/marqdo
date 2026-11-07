/* No network dependencies: this module also runs from a file:// static export. */
(function () {
  'use strict';
  var dialog, content, returnNodes = [], pending = false, toastTimer;
  function toast(message) {
    var el = document.querySelector('.reader-toast');
    el.textContent = message; el.hidden = false;
    clearTimeout(toastTimer); toastTimer = setTimeout(function () { el.hidden = true; }, 4200);
  }
  function restorePanels() {
    returnNodes.forEach(function (item) { if (item.marker.parentNode) item.marker.replaceWith(item.node); });
    returnNodes = [];
    document.querySelectorAll('.reader-tabs button').forEach(function (b) {
      var active = b.dataset.readerAction === 'document';
      b.classList.toggle('is-active', active);
      if (active) b.setAttribute('aria-current', 'page'); else b.removeAttribute('aria-current');
    });
  }
  function openDialog(title, action) {
    restorePanels(); content.replaceChildren();
    document.getElementById('reader-dialog-title').textContent = title;
    if (!dialog.open) dialog.showModal();
    document.querySelectorAll('.reader-tabs button').forEach(function (b) {
      var active = b.dataset.readerAction === action;
      b.classList.toggle('is-active', active);
      if (active) b.setAttribute('aria-current', 'page'); else b.removeAttribute('aria-current');
    });
  }
  function movePanel(node) {
    if (!node) { content.textContent = '当前文档暂无此类信息。'; return; }
    var marker = document.createComment('reader-panel'); node.before(marker);
    returnNodes.push({ node: node, marker: marker }); content.appendChild(node);
  }
  function option(label, callback) {
    var b = document.createElement('button'); b.type = 'button'; b.className = 'reader-option';
    b.textContent = label; b.addEventListener('click', callback); content.appendChild(b); return b;
  }
  function clickExisting(id) {
    var el = document.getElementById(id);
    if (el && !el.disabled) el.click(); else toast('当前文档没有可用的此项操作。');
  }
  function scrollTo(el) {
    if (!el) { toast('当前文档暂无此类信息。'); return; }
    if (dialog.open) dialog.close();
    el.scrollIntoView({ behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth', block: 'center' });
  }
  function showSearch() {
    openDialog('搜索文档', 'search');
    var input = document.createElement('input'); input.type = 'search'; input.placeholder = '搜索正文、函数或文件…'; input.setAttribute('aria-label', '搜索内容');
    var results = document.createElement('div'); results.setAttribute('aria-live', 'polite'); content.append(input, results);
    input.addEventListener('input', function () {
      results.replaceChildren(); var q = input.value.trim().toLocaleLowerCase(); if (!q) return;
      var nodes = Array.from(document.querySelectorAll('.document-surface .doc-prose, .doc-fun-title, .nav a.file'));
      var count = 0;
      nodes.forEach(function (node) {
        if (count >= 40 || node.textContent.toLocaleLowerCase().indexOf(q) < 0) return;
        count++;
        var b = document.createElement('button'); b.type = 'button'; b.className = 'reader-option';
        b.textContent = node.textContent.trim().slice(0, 150);
        b.addEventListener('click', function () { if (node.matches('a')) location.href = node.href; else scrollTo(node); }); results.appendChild(b);
      });
      if (!count) results.textContent = '没有找到匹配内容。';
    });
    input.focus();
  }
  function action(name) {
    if (name === 'files') {
      var toggle = document.getElementById('nav-toggle'); toggle.checked = !toggle.checked;
      document.querySelector('.reader-menu').setAttribute('aria-expanded', String(toggle.checked));
      if (toggle.checked) { var first = document.querySelector('.nav a'); if (first) first.focus(); } return;
    }
    if (name === 'close-dialog') { dialog.close(); return; }
    if (name === 'document') { if (dialog.open) dialog.close(); scrollTo(document.getElementById('document-section') || document.querySelector('main')); return; }
    if (name === 'search') { showSearch(); return; }
    if (name === 'functions' || name === 'variables') {
      var panel = document.getElementById(name === 'functions' ? 'fn-outline' : 'vars-panel');
      openDialog(name === 'functions' ? '函数目录' : '文档变量', name); movePanel(panel); return;
    }
    if (name === 'memory') {
      openDialog('执行内存与调用轨迹', name);
      var hint = document.createElement('p'); hint.textContent = window.MARQDO_VIEW_LIVE ? '当前文档已捕获的绑定与调用轨迹；会话状态可在高级工具中查看。' : '以下为导出时保存的执行快照。'; content.appendChild(hint);
      var pre = document.createElement('pre');
      try { pre.textContent = JSON.stringify(JSON.parse((document.getElementById('mq-view-trace') || {}).textContent || '{}'), null, 2); }
      catch (_) { pre.textContent = '当前没有可用执行快照。'; }
      content.appendChild(pre); return;
    }
    if (name === 'execution') { scrollTo(document.getElementById('exec-block')); return; }
    if (name === 'run') {
      if (!window.MARQDO_VIEW_LIVE) { toast('静态页面展示导出时的结果。请使用 marqdo view 启动实时运行。'); return; }
      clickExisting(document.getElementById('stream-run') ? 'stream-run' : 'nb-run-all'); toast('已请求运行文档。'); return;
    }
    if (name === 'reload') { location.reload(); return; }
    if (name === 'clear') {
      if (window.MARQDO_VIEW_LIVE) clickExisting('nb-clear');
      else { document.querySelectorAll('.code-out, #exec-main').forEach(function (el) { el.hidden = true; }); toast('已隐藏输出，重新加载可恢复导出快照。'); }
      return;
    }
    if (name === 'settings') {
      openDialog('阅读设置', name);
      option('切换浅色 / 深色', function () { clickExisting('nb-theme'); });
      option('切换标准 / 大号正文', function () { document.body.classList.toggle('reader-large'); refreshAnnotations(); });
      option('显示 / 隐藏执行标注', function () { document.body.classList.toggle('reader-hide-notes'); refreshAnnotations(); });
      return;
    }
    if (name === 'more') {
      openDialog('文档工具', name);
      option('文件目录', function () { dialog.close(); action('files'); });
      option('查看完整源码', function () { var source = document.querySelector('.source-block'); if (source) { source.open = true; scrollTo(source); } });
      option('高级运行工具', function () { document.body.classList.toggle('reader-tools'); dialog.close(); scrollTo(document.getElementById('notebook-bar')); });
      option('打印 / 保存为 PDF', function () { dialog.close(); window.print(); });
    }
  }
  function findAnchor(block, chip) {
    var id = chip.dataset.annotId;
    var anchors = block.querySelectorAll('.mq-anchor');
    for (var i = 0; i < anchors.length; i++) if (anchors[i].dataset.annotId === id) return anchors[i];
    var kind = chip.dataset.annotKind;
    return block.querySelector(kind === 'bind' ? '.mq-decl' : kind === 'return' ? '.mq-return' : '.mq-code') || block.querySelector('.doc-line, .doc-prose') || block;
  }
  function layoutAnnotations() {
    var bands = document.querySelectorAll('[data-annot-band]');
    // Reset geometry first; fallback is ordinary flow with all controls preserved.
    bands.forEach(function (band) {
      var block = band.closest('.doc-block'); if (!block) return;
      block.classList.remove('has-margin-notes', 'is-keyed'); band.classList.remove('is-keyed');
      block.querySelectorAll(':scope > .annot-leaders').forEach(function (el) { el.replaceChildren(); });
      band.querySelectorAll('.annot-chip').forEach(function (chip) { chip.classList.remove('is-margin-note'); chip.style.removeProperty('left'); chip.style.removeProperty('top'); });
    });
    var surface = document.querySelector('.document-surface');
    if (!surface || innerWidth <= 1000 || document.body.classList.contains('reader-hide-notes') || matchMedia('print').matches) return;
    var bounds = surface.getBoundingClientRect(), nextY = [bounds.top + scrollY, bounds.top + scrollY];
    if (bounds.left < 180 || innerWidth - bounds.right < 180) return;
    bands.forEach(function (band) {
      var block = band.closest('.doc-block'); if (!block || !block.getClientRects().length) return;
      var chips = Array.from(band.querySelectorAll('.annot-chip')).filter(function (c) { return !c.hidden && !c.classList.contains('is-frame-hidden') && (!c.classList.contains('annot-chip-overflow') || band.classList.contains('is-expanded')); });
      if (!chips.length || chips.length > 4 || !band.querySelector('[data-annot-more][hidden]') && band.querySelector('[data-annot-more]')) return;
      var rect = block.getBoundingClientRect(), savedY = nextY.slice(), placed = [];
      block.classList.add('has-margin-notes');
      chips.forEach(function (chip) {
        var anchor = findAnchor(block, chip), a = anchor.getBoundingClientRect();
        var side = nextY[0] <= nextY[1] ? 0 : 1;
        var y = Math.max(a.top + scrollY - 4, nextY[side]);
        var left = side === 0 ? bounds.left - rect.left - 180 : bounds.right - rect.left + 30;
        chip.classList.add('is-margin-note'); chip.style.left = left + 'px'; chip.style.top = (y - scrollY - rect.top) + 'px';
        nextY[side] = y + chip.offsetHeight + 20;
        placed.push({ chip:chip, anchor:a, side:side, y:y });
      });
      // If dense notes drift too far away from their statement, retain inline notes.
      if (placed.some(function (p) { return p.y - (p.anchor.top + scrollY) > 140; })) {
        nextY = savedY; block.classList.remove('has-margin-notes');
        chips.forEach(function (c) { c.classList.remove('is-margin-note'); c.style.removeProperty('left'); c.style.removeProperty('top'); }); return;
      }
      var leaders = block.querySelector(':scope > .annot-leaders');
      if (!leaders) { leaders = document.createElement('div'); leaders.className = 'annot-leaders'; leaders.setAttribute('aria-hidden','true'); block.prepend(leaders); }
      placed.forEach(function (p) {
        var c = p.chip.getBoundingClientRect(), a = p.anchor;
        var x0 = p.side === 0 ? c.right + 7 : bounds.right + 4, y0 = p.side === 0 ? c.top + 15 : a.top + a.height/2;
        var x1 = p.side === 0 ? bounds.left - 4 : c.left - 7, y1 = p.side === 0 ? a.top + a.height/2 : c.top + 15;
        var line = document.createElement('span'); line.className = 'reader-leader';
        line.style.left = (x0-rect.left)+'px'; line.style.top = (y0-rect.top)+'px';
        line.style.width = Math.hypot(x1-x0,y1-y0)+'px'; line.style.transform = 'rotate('+Math.atan2(y1-y0,x1-x0)+'rad)'; leaders.appendChild(line);
      });
    });
  }
  function refreshAnnotations() {
    if (pending) return; pending = true;
    requestAnimationFrame(function () { pending = false; layoutAnnotations(); });
  }
  function applyExecution(data) {
    if (!data.ok) return;
    var token = window.MARQDO_VIEW_PAGE_TOKEN || '';
    if (token && data.page_token && token !== data.page_token) { toast('文档已修改，请重新加载以查看新执行信息。'); return; }
    var trace = document.getElementById('mq-view-trace');
    if (trace && data.trace_json) trace.textContent = data.trace_json;
    var structure = document.getElementById('structure');
    var choices = [];
    if (structure && data.structure_html) {
      structure.querySelectorAll('.call-picker,.loop-picker').forEach(function (el, i) { choices.push({index:i,value:el.value}); });
      structure.innerHTML = data.structure_html;
      if (window.marqdoInitDocument) window.marqdoInitDocument();
      var pickers = structure.querySelectorAll('.call-picker,.loop-picker');
      choices.forEach(function (choice) {
        var el=pickers[choice.index]; if(el && Array.from(el.options).some(function(o){return o.value===choice.value;})) {el.value=choice.value;el.dispatchEvent(new Event('change',{bubbles:true}));}
      });
      if (window.renderMathInElement) window.renderMathInElement(structure, {delimiters:[{left:'$$',right:'$$',display:true},{left:'\\(',right:'\\)',display:false}]});
    }
    var vars = document.getElementById('vars-panel'); if (vars && typeof data.bindings_html === 'string') vars.innerHTML = data.bindings_html;
    refreshAnnotations();
  }
  window.MarqdoReader = { refreshAnnotations: refreshAnnotations, applyExecution: applyExecution };
  function init() {
    dialog = document.getElementById('reader-dialog'); content = document.getElementById('reader-dialog-content');
    dialog.addEventListener('close', function () { restorePanels(); refreshAnnotations(); });
    document.addEventListener('click', function (ev) {
      var button = ev.target.closest('[data-reader-action]'); if (button) action(button.dataset.readerAction);
      var chip = ev.target.closest('.annot-chip');
      if (chip) { openDialog(chip.querySelector('.annot-label').textContent, 'annotation'); var pre = document.createElement('pre'); pre.textContent = chip.title || chip.textContent; content.appendChild(pre); }
      if (ev.target.closest('.reader-brand')) { ev.preventDefault(); window.scrollTo({top:0,behavior:'auto'}); }
      if (dialog.open && ev.target.closest('.outline-panel a')) { dialog.close(); }
    });
    function focusNote(ev, focus) {
      var chip = ev.target.closest('.annot-chip'); if (!chip) return;
      var block = chip.closest('.doc-block'); if (!block) return;
      findAnchor(block,chip).classList.toggle('is-focus', focus); chip.classList.toggle('is-focus',focus);
    }
    document.addEventListener('pointerover',function(e){focusNote(e,true);});
    document.addEventListener('pointerout',function(e){focusNote(e,false);});
    document.addEventListener('focusin',function(e){focusNote(e,true);});
    document.addEventListener('focusout',function(e){focusNote(e,false);});
    document.addEventListener('keydown',function(e){
      if(e.key==='Escape'){ var inspector=document.getElementById('var-dialog'); if(inspector && inspector.open) return; if(dialog.open){e.preventDefault();dialog.close();} document.getElementById('nav-toggle').checked=false; document.querySelector('.reader-menu').setAttribute('aria-expanded','false'); }
      if((e.ctrlKey||e.metaKey)&&e.key==='k'){ e.preventDefault(); showSearch(); }
    });
    document.addEventListener('change',refreshAnnotations);
    var path = window.MARQDO_FILE_PATH || '';
    if (/(^|\/)(00[-_])?(welcome|欢迎)/i.test(path)) {
      document.body.classList.add('reader-welcome');
      var hero = document.createElement('section'); hero.className = 'reader-hero'; hero.setAttribute('aria-label','Welcome to Marqdo');
      hero.innerHTML = '<p class="eyebrow">WELCOME TO</p><h1>Marqdo</h1><p class="reader-motto">代码即文档 · 文档即知识库 · 知识即可执行</p>';
      document.querySelector('main.panel').prepend(hero);
    }
    // Observe replacement after live runs, not our own leader/position writes.
    var surface = document.getElementById('structure');
    if (surface) {
      new MutationObserver(function (changes) {
        if (changes.some(function(m){ return Array.from(m.addedNodes).concat(Array.from(m.removedNodes)).some(function(n){return n.nodeType===1 && !n.matches('.reader-leader,.annot-leaders');}); })) refreshAnnotations();
      }).observe(surface,{childList:true,subtree:true});
      if (typeof ResizeObserver !== 'undefined') new ResizeObserver(refreshAnnotations).observe(surface);
    }
    window.addEventListener('resize',refreshAnnotations);
    window.addEventListener('beforeprint',layoutAnnotations); window.addEventListener('afterprint',refreshAnnotations);
    if(document.fonts) document.fonts.ready.then(refreshAnnotations);
    refreshAnnotations();
  }
  if(document.readyState==='loading') document.addEventListener('DOMContentLoaded',init); else init();
})();
