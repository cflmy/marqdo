/**
 * 分段板块栏：横向滚动 + 滑动光晕（首页与列表页共用）
 */
function initFeedBoardUI(root) {
    if (!root) return;

    const scrollWrap = root.querySelector('.home-posts-tabs-scroll, .feed-tabs-scroll');
    const scrollLeft = root.querySelector('.home-posts-scroll-left, .feed-scroll-left');
    const scrollRight = root.querySelector('.home-posts-scroll-right, .feed-scroll-right');
    const spotlight = root.querySelector('.home-posts-spotlight, .feed-spotlight');
    const segment = root.querySelector('.home-posts-segment, .feed-segment');

    if (scrollWrap && scrollLeft && scrollRight) {
        const scrollAmount = 140;
        scrollLeft.addEventListener('click', function () {
            scrollWrap.scrollBy({ left: -scrollAmount, behavior: 'smooth' });
        });
        scrollRight.addEventListener('click', function () {
            scrollWrap.scrollBy({ left: scrollAmount, behavior: 'smooth' });
        });

        function updateScrollButtons() {
            const max = scrollWrap.scrollWidth - scrollWrap.clientWidth;
            const atStart = scrollWrap.scrollLeft <= 4;
            const atEnd = scrollWrap.scrollLeft >= max - 4;
            scrollLeft.classList.toggle('is-hidden', atStart);
            scrollRight.classList.toggle('is-hidden', atEnd);
        }

        updateScrollButtons();
        scrollWrap.addEventListener('scroll', updateScrollButtons);
        window.addEventListener('resize', updateScrollButtons);
    }

    if (spotlight && segment) {
        const tabSelector = '.home-posts-tab, .feed-tab';
        const moveSpotlight = (tab) => {
            if (!tab) return;
            const pad = segment.getBoundingClientRect();
            const rect = tab.getBoundingClientRect();
            const left = rect.left - pad.left;
            spotlight.style.setProperty('--spot-l', `${left}px`);
            spotlight.style.setProperty('--spot-w', `${rect.width}px`);
        };

        moveSpotlight(root.querySelector(`${tabSelector}.active`));

        root.querySelectorAll(tabSelector).forEach((tab) => {
            tab.addEventListener('click', () => {
                requestAnimationFrame(() => moveSpotlight(tab));
            });
        });

        window.addEventListener('resize', () => {
            moveSpotlight(root.querySelector(`${tabSelector}.active`));
        });
    }
}

document.addEventListener('DOMContentLoaded', function () {
    document.querySelectorAll('[data-feed-board]').forEach(initFeedBoardUI);
});
