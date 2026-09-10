/* 公聊客户端：保持极薄；业务配置仍在 .mq.md 表格里。 */
(function () {
  var log = document.getElementById("chat-log");
  var input = document.getElementById("chat-msg");
  var send = document.getElementById("chat-send");
  if (!log || !input || !send) return;

  var proto = location.protocol === "https:" ? "wss:" : "ws:";
  var ws = new WebSocket(proto + "//" + location.host + "/chat/ws");

  function line(text) {
    log.textContent += text + "\n";
    log.scrollTop = log.scrollHeight;
  }

  ws.onopen = function () {
    line("[已连接公聊室]");
  };
  ws.onclose = function () {
    line("[连接关闭]");
  };
  ws.onmessage = function (ev) {
    line(String(ev.data));
  };

  function publish() {
    var t = (input.value || "").trim();
    if (!t || ws.readyState !== 1) return;
    ws.send(t);
    input.value = "";
  }

  send.addEventListener("click", publish);
  input.addEventListener("keydown", function (e) {
    if (e.key === "Enter") publish();
  });
})();
