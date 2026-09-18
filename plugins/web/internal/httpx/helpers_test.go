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

func TestPathParamNamesIncludesID(t *testing.T) {
	got := pathParamNames("/desk/posts/{id}")
	if len(got) != 1 || got[0] != "id" {
		t.Fatalf("got %#v", got)
	}
	got = pathParamNames("/post/{slug}")
	if len(got) != 1 || got[0] != "slug" {
		t.Fatalf("got %#v", got)
	}
}

