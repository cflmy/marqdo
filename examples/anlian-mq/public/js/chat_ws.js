(function () {
    'use strict';

    function wsBaseUrl() {
        var scheme = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        return scheme + '//' + window.location.host;
    }

    function escapeHtml(text) {
        var div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    function formatContent(text) {
        return escapeHtml(text).replace(/\n/g, '<br>');
    }

    function scrollToBottom(container) {
        if (!container) {
            return;
        }
        container.scrollTop = container.scrollHeight;
    }

    function removeEmptyHint(container) {
        var empty = container.querySelector('.chat-empty');
        if (empty) {
            empty.remove();
        }
    }

    function buildRoomMessageNode(data, profileBase) {
        var article = document.createElement('article');
        article.className = 'chat-message' + (data.is_mine ? ' chat-message-mine' : '');
        article.dataset.messageId = String(data.id);

        var profileLink = '';
        if (!data.is_anonymous && data.author_username) {
            profileLink =
                '<a class="chat-profile-link" href="' +
                profileBase +
                encodeURIComponent(data.author_username) +
                '/">主页</a>';
        }

        article.innerHTML =
            '<img class="chat-avatar" src="' +
            escapeHtml(data.author_avatar_url) +
            '" alt="" width="40" height="40" loading="lazy" decoding="async">' +
            '<div class="chat-message-body">' +
            '<header class="chat-message-meta">' +
            '<strong class="chat-author">' +
            escapeHtml(data.author_display) +
            '</strong>' +
            profileLink +
            '<time datetime="' +
            escapeHtml(data.create_time_iso || '') +
            '">' +
            escapeHtml(data.create_time || '') +
            '</time>' +
            '</header>' +
            '<p class="chat-content">' +
            formatContent(data.content) +
            '</p>' +
            '</div>';

        return article;
    }

    function buildDirectMessageNode(data) {
        var article = document.createElement('article');
        article.className = 'chat-message' + (data.is_mine ? ' chat-message-mine' : '');
        article.dataset.messageId = String(data.id);

        article.innerHTML =
            '<img class="chat-avatar" src="' +
            escapeHtml(data.author_avatar_url) +
            '" alt="" width="40" height="40" loading="lazy" decoding="async">' +
            '<div class="chat-message-body">' +
            '<header class="chat-message-meta">' +
            '<strong class="chat-author">' +
            escapeHtml(data.author_display) +
            '</strong>' +
            '<time datetime="' +
            escapeHtml(data.create_time_iso || '') +
            '">' +
            escapeHtml(data.create_time || '') +
            '</time>' +
            '</header>' +
            '<p class="chat-content">' +
            formatContent(data.content) +
            '</p>' +
            '</div>';

        return article;
    }

    function appendMessage(container, node) {
        removeEmptyHint(container);
        if (container.querySelector('[data-message-id="' + node.dataset.messageId + '"]')) {
            return;
        }
        container.appendChild(node);
        scrollToBottom(container);
    }

    function setStatus(root, text, isError) {
        var status = root.querySelector('.chat-ws-status');
        if (!status) {
            return;
        }
        status.textContent = text;
        status.classList.toggle('chat-ws-status-error', !!isError);
        status.classList.toggle('chat-ws-status-ok', !isError);
    }

    function initChatApp(root) {
        var wsPath = root.dataset.wsPath;
        var kind = root.dataset.chatKind || 'room';
        var canSend = root.dataset.canSend === '1';
        var profileBase = root.dataset.profileBase || '/user/';
        var messagesEl = root.querySelector('#chat-messages');
        var form = root.querySelector('.chat-compose');
        var textarea = form ? form.querySelector('textarea[name="content"]') : null;
        var anonInput = form ? form.querySelector('input[name="is_anonymous"]') : null;
        var errorBox = root.querySelector('.chat-ws-errors');
        var socket = null;
        var reconnectTimer = null;
        var reconnectDelay = 1000;
        var reconnectAttempts = 0;
        var maxReconnectAttempts = 12;

        if (!wsPath || !messagesEl) {
            return;
        }

        function showError(detail) {
            if (!errorBox) {
                return;
            }
            errorBox.textContent = detail;
            errorBox.hidden = false;
        }

        function clearError() {
            if (errorBox) {
                errorBox.hidden = true;
                errorBox.textContent = '';
            }
        }

        function connect() {
            if (socket && (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING)) {
                return;
            }

            setStatus(root, '连接中…', false);
            socket = new WebSocket(wsBaseUrl() + wsPath);

            socket.addEventListener('open', function () {
                reconnectDelay = 1000;
                reconnectAttempts = 0;
                setStatus(root, '已连接', false);
                clearError();
            });

            socket.addEventListener('message', function (event) {
                var payload;
                try {
                    payload = JSON.parse(event.data);
                } catch (err) {
                    return;
                }

                if (payload.event === 'connected') {
                    setStatus(root, '已连接', false);
                    return;
                }

                if (payload.event === 'error') {
                    showError(payload.detail || '发送失败');
                    return;
                }

                if (payload.event === 'message' && payload.message) {
                    clearError();
                    var node =
                        kind === 'direct'
                            ? buildDirectMessageNode(payload.message)
                            : buildRoomMessageNode(payload.message, profileBase);
                    appendMessage(messagesEl, node);
                }
            });

            socket.addEventListener('close', function () {
                socket = null;
                reconnectAttempts += 1;
                if (reconnectAttempts >= maxReconnectAttempts) {
                    setStatus(root, '连接失败', true);
                    showError('无法连接聊天服务器，请刷新页面后重试');
                    return;
                }
                setStatus(root, '连接已断开，正在重连…', true);
                reconnectTimer = window.setTimeout(function () {
                    reconnectDelay = Math.min(reconnectDelay * 2, 15000);
                    connect();
                }, reconnectDelay);
            });

            socket.addEventListener('error', function () {
                setStatus(root, '连接异常', true);
            });
        }

        if (form && canSend) {
            form.addEventListener('submit', function (event) {
                event.preventDefault();
                clearError();

                if (!socket || socket.readyState !== WebSocket.OPEN) {
                    showError('尚未连接到服务器，请稍候再试');
                    connect();
                    return;
                }

                var content = (textarea && textarea.value ? textarea.value : '').trim();
                if (!content) {
                    showError('消息不能为空');
                    return;
                }

                socket.send(
                    JSON.stringify({
                        action: 'send',
                        content: content,
                        is_anonymous: !!(anonInput && anonInput.checked),
                    })
                );
                if (textarea) {
                    textarea.value = '';
                }
            });
        }

        scrollToBottom(messagesEl);

        function scheduleConnect() {
            var started = false;
            function run() {
                if (started) {
                    return;
                }
                started = true;
                connect();
            }
            if (form) {
                form.addEventListener('focusin', run, { once: true });
            }
            if ('requestIdleCallback' in window) {
                requestIdleCallback(run, { timeout: 2500 });
            } else {
                setTimeout(run, 100);
            }
        }

        setStatus(root, '待连接', false);
        scheduleConnect();

        window.addEventListener('beforeunload', function () {
            if (reconnectTimer) {
                window.clearTimeout(reconnectTimer);
            }
            if (socket) {
                socket.close();
            }
        });
    }

    document.addEventListener('DOMContentLoaded', function () {
        var root = document.getElementById('chat-app');
        if (root) {
            initChatApp(root);
        }
    });
})();
