// 实时 WebSocket 小部件（连接 /live 端点）。
// 连接 ws://host/live，发送输入，展示服务器回显。
(function () {
  var out = document.getElementById("live-out");
  var input = document.getElementById("live-msg");
  var btn = document.getElementById("live-send");
  var status = document.getElementById("live-status");
  if (!out || !input || !btn || !status) return;

  var proto = location.protocol === "https:" ? "wss" : "ws";
  var ws = new WebSocket(proto + "://" + location.host + "/live");
  status.textContent = "connecting…";

  ws.onopen = function () {
    status.textContent = "connected";
    btn.disabled = false;
  };
  ws.onmessage = function (ev) {
    var p = document.createElement("p");
    p.textContent = "echo: " + ev.data;
    out.appendChild(p);
    out.scrollTop = out.scrollHeight;
  };
  ws.onclose = function () {
    status.textContent = "closed";
    btn.disabled = true;
  };
  ws.onerror = function () {
    status.textContent = "error";
    btn.disabled = true;
  };

  btn.onclick = function () {
    var m = input.value.trim();
    if (m) ws.send(m);
  };
  input.addEventListener("keydown", function (e) {
    if (e.key === "Enter") btn.onclick();
  });
})();
