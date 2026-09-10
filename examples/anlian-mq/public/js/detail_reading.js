(function () {
  'use strict';

  function perView() {
    return window.matchMedia('(min-width: 768px)').matches ? 2 : 1;
  }

  function pageCount(total, pv) {
    if (total <= 0) return 0;
    return Math.max(1, total - pv + 1);
  }

  function initCarousel(root) {
    var track = root.querySelector('.detail-recommend-track');
    var dotsWrap = root.querySelector('.detail-recommend-dots');
    if (!track || !dotsWrap) return;

    var slides = track.querySelectorAll('.detail-recommend-slide');
    var total = slides.length;
    if (total === 0) return;

    var index = 0;
    var timer = null;
    var pv = perView();
    var pages = pageCount(total, pv);

    function renderDots() {
      dotsWrap.innerHTML = '';
      for (var i = 0; i < pages; i++) {
        var btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'detail-recommend-dot' + (i === index ? ' is-active' : '');
        btn.setAttribute('aria-label', '第 ' + (i + 1) + ' 组推荐');
        btn.setAttribute('aria-selected', i === index ? 'true' : 'false');
        btn.dataset.page = String(i);
        btn.addEventListener('click', function () {
          goTo(parseInt(this.dataset.page, 10));
          restartAuto();
        });
        dotsWrap.appendChild(btn);
      }
    }

    function applyTransform() {
      var slidePct = 100 / pv;
      track.style.transform = 'translateX(-' + index * slidePct + '%)';
      var dotEls = dotsWrap.querySelectorAll('.detail-recommend-dot');
      dotEls.forEach(function (el, i) {
        el.classList.toggle('is-active', i === index);
        el.setAttribute('aria-selected', i === index ? 'true' : 'false');
      });
    }

    function goTo(i) {
      index = Math.max(0, Math.min(i, pages - 1));
      applyTransform();
    }

    function restartAuto() {
      if (timer) clearInterval(timer);
      if (pages <= 1) return;
      timer = setInterval(function () {
        goTo((index + 1) % pages);
      }, 5000);
    }

    function onResize() {
      var newPv = perView();
      if (newPv !== pv) {
        pv = newPv;
        pages = pageCount(total, pv);
        if (index >= pages) index = pages - 1;
        renderDots();
      }
      applyTransform();
    }

    renderDots();
    applyTransform();
    restartAuto();
    window.addEventListener('resize', onResize);

    root.addEventListener('mouseenter', function () {
      if (timer) clearInterval(timer);
    });
    root.addEventListener('mouseleave', restartAuto);
  }

  function boot() {
    document.querySelectorAll('[data-recommend-carousel]').forEach(initCarousel);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }
})();
