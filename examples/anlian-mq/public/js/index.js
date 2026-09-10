/**
 * 首页：新闻/帖子板块 AJAX（板块栏交互见 feed_board.js）
 */
document.addEventListener('DOMContentLoaded', function () {
    if (document.body.classList.contains('page-home')) {
        initHomePanelSwap();
    }
});

function initHomePanelSwap() {
    const newsPanel = document.getElementById('homeNewsPanel');
    const postPanel = document.getElementById('tenPostContainer');

    bindPanelTabs({
        tabSelector: '.js-home-news-tab',
        panel: newsPanel,
        queryKey: 'news_board_id',
        getBoardId: (el) => el.dataset.newsBoard || '0',
        onAfterActive: (tab) => syncFeedBoardSpotlight('.home-news-board-bar', tab),
    });

    bindPanelTabs({
        tabSelector: '.js-home-post-tab',
        panel: postPanel,
        queryKey: 'board_id',
        getBoardId: (el) => el.dataset.board || '0',
        onAfterActive: (tab) => syncFeedBoardSpotlight('.home-posts-board-bar', tab),
    });
}

function bindPanelTabs({ tabSelector, panel, queryKey, getBoardId, onSwapped, onAfterActive }) {
    if (!panel) return;

    document.addEventListener('click', function (e) {
        const tab = e.target.closest(tabSelector);
        if (!tab) return;

        e.preventDefault();

        if (tab.classList.contains('active')) return;

        const boardId = getBoardId(tab);
        const requestUrl = tab.getAttribute('href');

        setActiveTab(tabSelector, tab);
        if (typeof onAfterActive === 'function') {
            onAfterActive(tab);
        }
        swapPanelHtml(panel, requestUrl, onSwapped);
        updateHomeUrlParam(queryKey, boardId);
    });
}

function setActiveTab(tabSelector, activeTab) {
    const root = activeTab.closest('.home-posts-tabs');
    if (!root) return;
    root.querySelectorAll(tabSelector).forEach((el) => el.classList.remove('active'));
    activeTab.classList.add('active');
}

function syncFeedBoardSpotlight(barSelector, tab) {
    const root = document.querySelector(barSelector);
    if (!root || !tab) return;
    const spotlight = root.querySelector('.home-posts-spotlight');
    const segment = root.querySelector('.home-posts-segment');
    if (!spotlight || !segment) return;
    const pad = segment.getBoundingClientRect();
    const rect = tab.getBoundingClientRect();
    spotlight.style.setProperty('--spot-l', `${rect.left - pad.left}px`);
    spotlight.style.setProperty('--spot-w', `${rect.width}px`);
}

async function swapPanelHtml(panel, url, onSwapped) {
    try {
        const response = await fetch(url, {
            headers: { 'X-Requested-With': 'XMLHttpRequest' },
        });
        if (!response.ok) throw new Error(response.status);
        const html = await response.text();
        panel.innerHTML = html;
        if (typeof onSwapped === 'function') {
            onSwapped(panel);
        }
    } catch (err) {
        console.error('板块内容加载失败', err);
        panel.innerHTML =
            '<p class="home-panel-error text-muted text-center py-3">加载失败，请稍后重试</p>';
    }
}

function updateHomeUrlParam(key, value) {
    const url = new URL(window.location.href);
    if (value === '0') {
        url.searchParams.delete(key);
    } else {
        url.searchParams.set(key, value);
    }
    history.replaceState(null, '', url.pathname + url.search);
}
