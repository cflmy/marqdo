(function () {
    'use strict';

    document.addEventListener('DOMContentLoaded', function () {
        var input = document.getElementById('chat-room-search-input');
        var grid = document.getElementById('chat-room-grid');
        var emptyHint = document.getElementById('chat-search-empty');
        if (!input || !grid) {
            return;
        }

        var cards = grid.querySelectorAll('.chat-room-card');

        function filterRooms() {
            var query = input.value.trim().toLowerCase();
            var visibleCount = 0;

            cards.forEach(function (card) {
                var haystack = (card.dataset.search || '').toLowerCase();
                var match = !query || haystack.indexOf(query) !== -1;
                card.hidden = !match;
                if (match) {
                    visibleCount += 1;
                }
            });

            if (emptyHint) {
                emptyHint.hidden = visibleCount > 0 || !query;
            }
        }

        input.addEventListener('input', filterRooms);
        filterRooms();
    });
})();
