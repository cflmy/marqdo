/**
 * 列表页局部刷新：板块切换与分页，无全页重载、无加载动画。
 */
(function () {
    const AJAX_HEADERS = { 'X-Requested-With': 'XMLHttpRequest' };

    document.addEventListener('DOMContentLoaded', function () {
        initFeedListPartial();
        initCommentListPartial();
    });

    function initFeedListPartial() {
        const page = document.querySelector('.feed-list-page[data-partial-feed]');
        if (!page) return;

        page.addEventListener('click', function (event) {
            const tab = event.target.closest('.feed-tab');
            const pageLink = event.target.closest('.feed-pagination .page-link[href]');
            const link = tab || pageLink;
            if (!link || link.classList.contains('disabled')) return;
            event.preventDefault();
            loadFeedPartial(page, link.href);
        });
    }

    async function loadFeedPartial(page, url) {
        const body = page.querySelector('.feed-list-body');
        if (!body) return;

        try {
            const response = await fetch(url, { headers: AJAX_HEADERS });
            if (!response.ok) throw new Error(String(response.status));
            const html = await response.text();
            const doc = new DOMParser().parseFromString(html, 'text/html');
            const newBody = doc.querySelector('.feed-list-body');
            const newPagination = doc.querySelector('.feed-pagination');
            if (newBody) body.innerHTML = newBody.innerHTML;

            let pagination = page.querySelector('.feed-pagination');
            if (newPagination) {
                if (pagination) {
                    pagination.replaceWith(newPagination);
                } else {
                    body.insertAdjacentElement('afterend', newPagination);
                }
                pagination = newPagination;
            } else if (pagination) {
                pagination.remove();
            }

            syncFeedTabsFromUrl(page, url);
            history.pushState(null, '', url);
            page.scrollIntoView({ block: 'nearest', behavior: 'instant' in window ? 'instant' : 'auto' });
        } catch (err) {
            console.error('列表局部刷新失败', err);
        }
    }

    function syncFeedTabsFromUrl(page, url) {
        const parsed = new URL(url, window.location.origin);
        const boardId = parsed.searchParams.get('board_id')
            || parsed.searchParams.get('news_board_id')
            || '';
        const param = parsed.searchParams.has('news_board_id') ? 'news_board_id' : 'board_id';

        page.querySelectorAll('.feed-tab').forEach((tab) => {
            const tabUrl = new URL(tab.href, window.location.origin);
            const tabBoard = tabUrl.searchParams.get(param) || '';
            const isActive = tabBoard === boardId;
            tab.classList.toggle('active', isActive);
        });

        const activeTab = page.querySelector('.feed-tab.active');
        const bar = page.querySelector('[data-feed-board]');
        if (activeTab && bar && typeof initFeedBoardUI === 'function') {
            initFeedBoardUI(bar);
        }
    }

    function initCommentListPartial() {
        const root = document.querySelector('[data-partial-comments]');
        if (!root) return;

        root.addEventListener('click', function (event) {
            const link = event.target.closest('.comment-pagination .page-link[href]');
            if (!link) return;
            event.preventDefault();
            loadCommentPartial(root, link.href);
        });
    }

    async function loadCommentPartial(root, url) {
        const list = root.querySelector('.comment-list-container');
        const pagination = root.querySelector('.comment-pagination');
        if (!list) return;

        try {
            const response = await fetch(url, { headers: AJAX_HEADERS });
            if (!response.ok) throw new Error(String(response.status));
            const html = await response.text();
            const doc = new DOMParser().parseFromString(html, 'text/html');
            const newList = doc.querySelector('.comment-list-container');
            const newPagination = doc.querySelector('.comment-pagination');
            if (newList) list.innerHTML = newList.innerHTML;
            if (newPagination && pagination) {
                pagination.replaceWith(newPagination);
            } else if (newPagination) {
                list.insertAdjacentElement('afterend', newPagination);
            } else if (pagination) {
                pagination.remove();
            }
            history.replaceState(null, '', url);
        } catch (err) {
            console.error('评论分页刷新失败', err);
        }
    }
})();
