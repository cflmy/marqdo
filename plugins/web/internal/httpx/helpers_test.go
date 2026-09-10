package httpx

import "testing"

func TestGoMuxPattern(t *testing.T) {
	if got := goMuxPattern("/_media/{*key}"); got != "/_media/{key...}" {
		t.Fatalf("got %q", got)
	}
	if got := goMuxPattern("/post/{slug}"); got != "/post/{slug}" {
		t.Fatalf("got %q", got)
	}
}
