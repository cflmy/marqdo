// Live widget for the /live WebSocket endpoint.
// Connects to ws://host/live, sends the input, shows the echo.
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
