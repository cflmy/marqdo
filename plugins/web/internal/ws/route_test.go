package ws_test

import (
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/ws"
)

func TestRouteWSBroadcastMode(t *testing.T) {
	a := app.New(map[string]any{"page": map[string]any{"title": "w6b"}})
	out, err := ws.RouteWS(a, "/room", map[string]any{"mode": "broadcast"})
	if err != nil {
		t.Fatal(err)
	}
	mode, err := ws.ModeOf(out, "/room")
	if err != nil {
		t.Fatal(err)
	}
	if mode != ws.ModeBroadcast {
		t.Fatalf("mode=%q want broadcast", mode)
	}
	routes := out["ws_routes"].(map[string]any)
	room := routes["/room"].(map[string]any)
	if room["mode"] != "broadcast" {
		t.Fatalf("bag mode=%v", room["mode"])
	}
}

func TestRouteWSEchoDefault(t *testing.T) {
	a := app.New(nil)
	out, err := ws.RouteWS(a, "/live", map[string]any{"echo": true})
	if err != nil {
		t.Fatal(err)
	}
	mode, err := ws.ModeOf(out, "/live")
	if err != nil {
		t.Fatal(err)
	}
	if mode != ws.ModeEcho {
		t.Fatalf("mode=%q want echo", mode)
	}
}

func TestRouteWSDrainFromEchoFalse(t *testing.T) {
	a := app.New(nil)
	out, err := ws.RouteWS(a, "/sink", map[string]any{"echo": false})
	if err != nil {
		t.Fatal(err)
	}
	mode, err := ws.ModeOf(out, "/sink")
	if err != nil {
		t.Fatal(err)
	}
	if mode != ws.ModeDrain {
		t.Fatalf("mode=%q want drain", mode)
	}
}

func TestParseModeVariants(t *testing.T) {
	cases := []struct {
		in   any
		want ws.Mode
	}{
		{true, ws.ModeEcho},
		{false, ws.ModeDrain},
		{"broadcast", ws.ModeBroadcast},
		{"广播", ws.ModeBroadcast},
		{"drain", ws.ModeDrain},
		{"nope", ws.ModeEcho},
	}
	for _, c := range cases {
		if got := ws.ParseMode(c.in); got != c.want {
			t.Fatalf("ParseMode(%#v)=%q want %q", c.in, got, c.want)
		}
	}
}

func TestConnectRefusesBadURL(t *testing.T) {
	out := ws.Connect("http://127.0.0.1:1", "hi", nil, 1)
	if out["ok"] == true {
		t.Fatal("expected ok=false for http url")
	}
	err, _ := out["error"].(string)
	if err == "" || !strings.Contains(err, "ws://") {
		t.Fatalf("error=%q", err)
	}
}
