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
  var anchorSequence = 0;
  function noteOwner(band) {
    return band ? band.closest('.doc-block, .doc-list-item') : null;
  }
  function findAnchor(owner, chip) {
    if (!owner) return null;
    var line = chip.dataset.anchorLine, start = Number(chip.dataset.anchorByteStart);
    var end = Number(chip.dataset.anchorByteEnd), id = chip.dataset.annotId;
    var anchors = Array.from(owner.querySelectorAll('.mq-anchor')).filter(function (el) {
      return el.closest('.doc-block, .doc-list-item') === owner;
    });
    var exact = anchors.find(function (el) { return id && el.dataset.annotId === id; });
    if (exact) return exact;
    var onLine = anchors.filter(function (el) { return line && el.dataset.line === line; });
    if (chip.hasAttribute('data-anchor-byte-start')) {
      var span = onLine.filter(function (el) {
        return el.hasAttribute('data-byte-start') && Number(el.dataset.byteStart) <= start
          && Number(el.dataset.byteEnd) > start;
      }).sort(function (x, y) {
        return (Number(x.dataset.byteEnd) - Number(x.dataset.byteStart))
          - (Number(y.dataset.byteEnd) - Number(y.dataset.byteStart));
      });
      if (span.length) return span[0];
      if (chip.hasAttribute('data-anchor-byte-end')) {
        var overlap = onLine.find(function (el) {
          return Number(el.dataset.byteStart) < end && Number(el.dataset.byteEnd) > start;
        });
        if (overlap) return overlap;
      }
    }
    // A branch chip belongs to this arm, not to the first arm's source line.
    if (owner.matches('.doc-list-item') && chip.dataset.annotKind === 'branch') {
      return owner.querySelector(':scope > .doc-list-head');
    }
    if (onLine.length === 1) return onLine[0];
    if (line && owner.dataset.line === line) {
      return owner.querySelector(':scope > .doc-list-head, :scope > .doc-line.doc-stmt, :scope > .doc-line.doc-occl');
    }
    // No guessed first declaration / whole paragraph: retain an inline result.
    return null;
  }
  function anchorFragments(anchor) {
    var rects;
    if (anchor.matches('.mq-anchor')) rects = Array.from(anchor.getClientRects());
    else {
      var range = document.createRange(); range.selectNodeContents(anchor);
      rects = Array.from(range.getClientRects());
    }
    return rects.filter(function (r) { return r.width > 1 && r.height > 1; });
  }
  function resetNotes(bands, surface) {
    bands.forEach(function (band) {
      var owner = noteOwner(band);
      if (owner) {
        owner.classList.remove('has-margin-notes', 'is-keyed');
        owner.querySelectorAll(':scope > .annot-leaders').forEach(function (el) { el.replaceChildren(); });
      }
      band.classList.remove('is-keyed');
      band.querySelectorAll('.annot-chip').forEach(function (chip) {
        chip.classList.remove('is-margin-note');
        ['left', 'top', 'width'].forEach(function (key) { chip.style.removeProperty(key); });
        delete chip.dataset.readerAnchor; delete chip.dataset.readerSide;
      });
    });
    if (surface) surface.querySelectorAll(':scope > .reader-connectors').forEach(function (el) { el.remove(); });
  }
  function setNoteTail(surface, height) {
    var value = Math.ceil(Math.max(0, height)) + 'px';
    if (surface.style.getPropertyValue('--reader-note-tail') !== value) surface.style.setProperty('--reader-note-tail', value);
  }
  function curvePoint(points, t) {
    var u = 1 - t;
    return {x:u*u*u*points[0].x + 3*u*u*t*points[1].x + 3*u*t*t*points[2].x + t*t*t*points[3].x,
      y:u*u*u*points[0].y + 3*u*u*t*points[1].y + 3*u*t*t*points[2].y + t*t*t*points[3].y};
  }
  function splitCurve(points) {
    function mid(a,b) { return {x:(a.x+b.x)/2,y:(a.y+b.y)/2}; }
    var a=mid(points[0],points[1]), b=mid(points[1],points[2]), c=mid(points[2],points[3]);
    var d=mid(a,b), e=mid(b,c), center=mid(d,e);
    return [[points[0],a,d,center],[center,e,c,points[3]]];
  }
  function curveData(curves) {
    var first=curves[0][0], path='M '+first.x+' '+first.y;
    curves.forEach(function (curve) {
      path+=' C '+curve[1].x+' '+curve[1].y+' '+curve[2].x+' '+curve[2].y+' '+curve[3].x+' '+curve[3].y;
    });
    return path;
  }
  function sourceObstacles(surface, inset, top) {
    var rects=[], walker=document.createTreeWalker(surface,NodeFilter.SHOW_TEXT);
    while (walker.nextNode()) {
      var text=walker.currentNode, parent=text.parentElement;
      if (!text.textContent.trim() || !parent || parent.closest('.annot-band,.annot-leaders,.reader-connectors,script,style')) continue;
      var range=document.createRange(); range.selectNodeContents(text);
      Array.from(range.getClientRects()).forEach(function (r) {
        if (r.width>1 && r.height>1) rects.push({left:r.left-inset-2,right:r.right-inset+2,top:r.top-top-1,bottom:r.bottom-top+1});
      });
    }
    surface.querySelectorAll('.doc-table,.code-card,.formula-card').forEach(function (el) {
      var r=el.getBoundingClientRect();
      rects.push({left:r.left-inset-3,right:r.right-inset+3,top:r.top-top-2,bottom:r.bottom-top+2});
    });
    return rects;
  }
  function clearCurve(points, obstacles) {
    var left=Math.min.apply(Math,points.map(function(p){return p.x;}));
    var right=Math.max.apply(Math,points.map(function(p){return p.x;}));
    var top=Math.min.apply(Math,points.map(function(p){return p.y;}));
    var bottom=Math.max.apply(Math,points.map(function(p){return p.y;}));
    var nearby=obstacles.filter(function(r){return r.left<right&&r.right>left&&r.top<bottom&&r.bottom>top;});
    for (var step=1;step<48;step++) {
      var p=curvePoint(points,step/48);
      if (nearby.some(function(r){return p.x>r.left&&p.x<r.right&&p.y>r.top&&p.y<r.bottom;})) return false;
    }
    return true;
  }
  function routeSamples(curves) {
    var samples=[];
    curves.forEach(function(curve,index){
      for(var step=index?1:0;step<=20;step++) samples.push(curvePoint(curve,step/20));
    });
    samples.bounds=samples.reduce(function(box,p){
      return {left:Math.min(box.left,p.x),right:Math.max(box.right,p.x),top:Math.min(box.top,p.y),bottom:Math.max(box.bottom,p.y)};
    },{left:Infinity,right:-Infinity,top:Infinity,bottom:-Infinity});
    return samples;
  }
  function clearRoutes(curves, routes) {
    var samples=routeSamples(curves);
    function cross(a,b,c){return (b.x-a.x)*(c.y-a.y)-(b.y-a.y)*(c.x-a.x);}
    function distance(p,a,b){
      var dx=b.x-a.x,dy=b.y-a.y, t=Math.max(0,Math.min(1,((p.x-a.x)*dx+(p.y-a.y)*dy)/(dx*dx+dy*dy||1)));
      return Math.pow(p.x-a.x-t*dx,2)+Math.pow(p.y-a.y-t*dy,2);
    }
    return routes.every(function(other){
      var box=samples.bounds, prior=other.bounds;
      if(box.right+8<prior.left||box.left-8>prior.right||box.bottom+8<prior.top||box.top-8>prior.bottom) return true;
      for(var i=1;i<samples.length;i++) for(var j=1;j<other.length;j++) {
        var a=samples[i-1],b=samples[i],c=other[j-1],d=other[j];
        if(Math.max(a.x,b.x)+8<Math.min(c.x,d.x)||Math.min(a.x,b.x)-8>Math.max(c.x,d.x)
          ||Math.max(a.y,b.y)+8<Math.min(c.y,d.y)||Math.min(a.y,b.y)-8>Math.max(c.y,d.y)) continue;
        if(cross(a,b,c)*cross(a,b,d)<0 && cross(c,d,a)*cross(c,d,b)<0) return false;
        var shared=Math.hypot(samples[0].x-other[0].x,samples[0].y-other[0].y)<3;
        if(shared && Math.hypot(a.x-samples[0].x,a.y-samples[0].y)<16
          && Math.hypot(c.x-other[0].x,c.y-other[0].y)<16) continue;
        if(Math.min(distance(a,c,d),distance(b,c,d),distance(c,a,b),distance(d,a,b))<64) return false;
      }
      return true;
    });
  }
  function flowingCurve(start, end, direction, bounds, inset, obstacles, routes) {
    var distance=Math.abs(end.x-start.x);
    var amplitude=Math.max(32,Math.min(128,distance*.24));
    var preferred=end.y>=start.y?-1:1;
    // Prefer one continuous, visible S-curve. Try the opposite sweep when a
    // paragraph occupies that space; never trade readable prose for ornament.
    for (var scale=1;scale>=.3;scale-=.35) {
      for (var signIndex=0;signIndex<2;signIndex++) {
        var wave=preferred*(signIndex===0?1:-1)*amplitude*scale;
        var points=[start,{x:start.x+direction*distance*.34,y:start.y+wave},
          {x:end.x-direction*distance*.34,y:end.y-wave},end];
        var curves=splitCurve(points);
        if (clearCurve(points,obstacles) && clearRoutes(curves,routes)) return curves;
      }
    }
    // In flowing paragraphs, keep a gently bowed passage beneath the source
    // line, then devote the free margin to a fuller curve. No straight rails.
    var edge={x:(direction<0?bounds.left-14:bounds.right+14)-inset,y:start.y+5};
    var innerDistance=Math.abs(edge.x-start.x), outerDistance=Math.abs(end.x-edge.x);
    var outerWave=Math.max(22,Math.min(84,outerDistance*.32));
    var fallback;
    for(var sweep=0;sweep<2;sweep++) {
      var wave=(sweep===0?1:-1)*outerWave;
      fallback=[[start,{x:start.x+direction*innerDistance*.28,y:start.y+8},
        {x:edge.x-direction*innerDistance*.28,y:edge.y},edge],
        [edge,{x:edge.x+direction*outerDistance*.34,y:edge.y},
          {x:end.x-direction*outerDistance*.34,y:end.y-wave},end]];
      if(fallback.every(function(curve){return clearCurve(curve,obstacles);}) && clearRoutes(fallback,routes)) return fallback;
    }
    return fallback;
  }

  function drawConnections(surface, bounds, inset, notes) {
    var ns = 'http://www.w3.org/2000/svg';
    var svg = document.createElementNS(ns, 'svg');
    svg.classList.add('reader-connectors'); svg.setAttribute('aria-hidden', 'true');
    svg.style.left = (inset - bounds.left) + 'px';
    svg.setAttribute('width', String(innerWidth - inset * 2));
    svg.setAttribute('height', String(surface.offsetHeight));
    var obstacles=sourceObstacles(surface,inset,bounds.top), routes=[];
    var defs=document.createElementNS(ns,'defs');svg.appendChild(defs);
    notes.forEach(function (note, noteIndex) {
      var fragments = anchorFragments(note.anchor);
      var a = note.side === 0 ? fragments[0] : fragments[fragments.length - 1];
      if (!a) return;
      var c = note.chip.getBoundingClientRect(), heading = note.chip.querySelector('.annot-label').getBoundingClientRect();
      var direction = note.side === 0 ? -1 : 1;
      var sx = (note.side === 0 ? a.left - 2 : a.right + 2) - inset, sy = a.bottom - bounds.top + 2;
      var ex = (note.side === 0 ? heading.right + 9 : heading.left - 9) - inset;
      var ey = heading.top + heading.height / 2 - bounds.top;
      var curves=flowingCurve({x:sx,y:sy},{x:ex,y:ey},direction,bounds,inset,obstacles,routes);
      routes.push(routeSamples(curves));
      var pathData=curveData(curves);
      var gradient=document.createElementNS(ns,'linearGradient');
      gradient.id='reader-curve-ink-'+noteIndex;gradient.setAttribute('gradientUnits','userSpaceOnUse');
      gradient.setAttribute('x1',sx);gradient.setAttribute('y1',sy);gradient.setAttribute('x2',ex);gradient.setAttribute('y2',ey);
      [['0%','reader-curve-gold'],['55%','reader-curve-olive'],['100%','reader-curve-gold']].forEach(function(stop){
        var node=document.createElementNS(ns,'stop');node.setAttribute('offset',stop[0]);node.setAttribute('class',stop[1]);gradient.appendChild(node);
      });
      defs.appendChild(gradient);
      var group = document.createElementNS(ns, 'g'); group.classList.add('reader-connector');
      group.dataset.annotId = note.chip.dataset.annotId;
      if (!note.anchor.dataset.readerSourceAnchor) note.anchor.dataset.readerSourceAnchor = 'reader-anchor-' + (++anchorSequence);
      group.dataset.readerAnchor = note.anchor.dataset.readerSourceAnchor;
      note.chip.dataset.readerAnchor = group.dataset.readerAnchor;
      group.dataset.startX = String(sx + inset); group.dataset.startY = String(sy + bounds.top);
      ['reader-leader-halo', 'reader-leader'].forEach(function (cls) {
        var path = document.createElementNS(ns, 'path'); path.setAttribute('class', cls); path.setAttribute('d', pathData);
        if(cls==='reader-leader') path.style.stroke='url(#'+gradient.id+')';
        group.appendChild(path);
      });
      [[sx, sy, 'reader-source-dot'], [ex, ey, 'reader-note-dot']].forEach(function (point) {
        var dot = document.createElementNS(ns, 'circle'); dot.setAttribute('cx', point[0]); dot.setAttribute('cy', point[1]);
        dot.setAttribute('r', point[2] === 'reader-source-dot' ? '2.2' : '1.7'); dot.setAttribute('class', point[2]); group.appendChild(dot);
      });
      svg.appendChild(group);
    });
    surface.appendChild(svg);
  }
  function layoutAnnotations() {
    var bands = Array.from(document.querySelectorAll('[data-annot-band]'));
    var surface = document.querySelector('.document-surface');
    resetNotes(bands, surface);
    if (!surface) return;
    if (innerWidth <= 1000 || document.body.classList.contains('reader-hide-notes') || matchMedia('print').matches) {
      setNoteTail(surface, 0); return;
    }
    var inset = Math.max(24, Math.min(56, innerWidth * .025));
    var bounds = surface.getBoundingClientRect();
    var space = [bounds.left - inset, innerWidth - inset - bounds.right];
    if (Math.min.apply(Math, space) < 180) { setNoteTail(surface, 0); return; }
    var eligible = bands.map(function (band) {
      var owner = noteOwner(band);
      if (!owner || !owner.getClientRects().length || owner.matches('.doc-table-wrap')) return null;
      var chips = Array.from(band.querySelectorAll('.annot-chip')).filter(function (chip) {
        return !chip.hidden && !chip.classList.contains('is-frame-hidden')
          && (!chip.classList.contains('annot-chip-overflow') || band.classList.contains('is-expanded'));
      });
      var more = band.querySelector('[data-annot-more]');
      if (!chips.length || chips.length > 4 || more && !more.hidden) return null;
      var items = chips.map(function (chip) { return {chip:chip, anchor:findAnchor(owner,chip), owner:owner, band:band}; });
      return items.every(function (it) { return it.anchor && anchorFragments(it.anchor).length; }) ? {band:band, owner:owner, items:items} : null;
    }).filter(Boolean);
    var excluded = new Set(), finalNotes = [];
    // Falling back to flow changes later source positions. Re-measure the whole
    // document after each fallback batch, instead of leaving stale endpoints.
    for (var pass = 0; pass <= eligible.length; pass++) {
      resetNotes(bands, surface);
      var active = eligible.filter(function (b) { return !excluded.has(b.band); });
      active.forEach(function (b) { b.owner.classList.add('has-margin-notes'); });
      bounds = surface.getBoundingClientRect();
      var nextY = [bounds.top + scrollY + 6, bounds.top + scrollY + 6], counts = [0,0];
      var items = [].concat.apply([], active.map(function (b) { return b.items; }));
      items.sort(function (a,b) { return a.anchor.getBoundingClientRect().top - b.anchor.getBoundingClientRect().top; });
      var rejected = new Set(), notes = [];
      items.forEach(function (item, index) {
        if (rejected.has(item.band)) return;
        var fragments = anchorFragments(item.anchor);
        var width = Math.max(140, Math.min(190, Math.min.apply(Math, space) * .58));
        item.chip.classList.add('is-margin-note'); item.chip.style.width = width + 'px';
        var height = item.chip.offsetHeight, stagger = [-58, 44, -34, 68][index % 4];
        var choices = nextY.map(function (y, side) {
          var fragment = side === 0 ? fragments[0] : fragments[fragments.length - 1];
          var desired = Math.max(bounds.top + scrollY + 6, fragment.bottom + scrollY - 16 + stagger);
          var top = Math.max(desired, y);
          return {side:side, anchor:fragment, top:top,
            cost:Math.max(0, y - desired) * 3 + counts[side] * 35 + (side === index % 2 ? 0 : 24)};
        });
        var choice = choices[0].cost <= choices[1].cost ? choices[0] : choices[1];
        var side = choice.side, top = choice.top;
        if (top - (choice.anchor.bottom + scrollY) > 170) { rejected.add(item.band); return; }
        var maxGap=space[side]-width;
        var baseGap=Math.min(130,space[side]*.24,maxGap-12);
        var gap=Math.max(24,Math.min(maxGap,baseGap+[0,28,-16,44][counts[side]%4]));
        var x=side===0?bounds.left-gap-width:bounds.right+gap;
        var ownerRect = item.owner.getBoundingClientRect();
        item.chip.style.left = (x - ownerRect.left) + 'px'; item.chip.style.top = (top - scrollY - ownerRect.top) + 'px';
        item.chip.dataset.readerSide = side === 0 ? 'left' : 'right';
        nextY[side] = top + height + 28; counts[side]++;
        notes.push({chip:item.chip,anchor:item.anchor,side:side});
      });
      if (rejected.size) { rejected.forEach(function (band) { excluded.add(band); }); continue; }
      finalNotes = notes; break;
    }
    var tail = parseFloat(surface.style.getPropertyValue('--reader-note-tail')) || 0;
    var bottom = bounds.bottom - tail;
    var noteBottom = finalNotes.reduce(function (max,note) { return Math.max(max,note.chip.getBoundingClientRect().bottom); }, bottom);
    setNoteTail(surface, noteBottom > bottom ? noteBottom - bottom + 20 : 0);
    if (finalNotes.length) drawConnections(surface, bounds, inset, finalNotes);
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
      var chip = ev.target.closest('.annot-chip'); if (!chip || ev.relatedTarget && chip.contains(ev.relatedTarget)) return;
      var owner = noteOwner(chip.closest('[data-annot-band]'));
      var anchor = findAnchor(owner,chip); if (anchor) anchor.classList.toggle('is-focus', focus);
      chip.classList.toggle('is-focus',focus);
      document.querySelectorAll('.reader-connector').forEach(function (group) {
        if (group.dataset.annotId === chip.dataset.annotId) group.classList.toggle('is-focus',focus);
      });
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
        if (changes.some(function(m){
          if (m.target.closest && m.target.closest('.reader-connectors')) return false;
          return Array.from(m.addedNodes).concat(Array.from(m.removedNodes)).some(function(n){
            return n.nodeType===1 && !n.matches('.reader-connectors,.reader-leader,.annot-leaders');
          });
        })) refreshAnnotations();
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
