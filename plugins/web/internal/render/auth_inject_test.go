package render

import (
	"strings"
	"testing"
)

func TestInjectAuthFieldsAddsCSRFAndNext(t *testing.T) {
	html := `<!DOCTYPE html><html><body><main>
<form id="site-login-form" method="post" action="/login">
<label>用户名<input name="username"/></label>
<button type="submit">登录</button>
</form>
<p id="site-auth-err" class="auth-err" hidden></p>
</main></body></html>`
	out := InjectAuthFields(html, "tok123", "/desk", "用户名或密码错误。")
	for _, want := range []string{`name="_csrf"`, `value="tok123"`, `name="next"`, `value="/desk"`, `用户名或密码错误`} {
		if !strings.Contains(out, want) {
			t.Fatalf("missing %q in %s", want, out)
		}
	}
	if strings.Contains(out, `id="site-auth-err" class="auth-err" hidden`) {
		t.Fatalf("err still hidden: %s", out)
	}
}

func TestInjectSkipsNonAuthForms(t *testing.T) {
	html := `<form method="post" action="/_form/comment"><input name="body"/></form>`
	out := InjectAuthFields(html, "tok", "/x", "")
	if strings.Contains(out, `_csrf`) {
		t.Fatalf("should not inject into comment form: %s", out)
	}
}

func TestRenderAuthFormHTML(t *testing.T) {
	spec := &AuthFormSpec{Action: "/desk/login", Submit: "进入后台", FormID: "desk-login-form", ErrID: "desk-login-err", NextDefault: "/desk", Kind: "login"}
	html := RenderAuthFormHTML(spec, "c", "", "")
	for _, want := range []string{`action="/desk/login"`, `name="_csrf"`, `value="/desk"`, `进入后台`} {
		if !strings.Contains(html, want) {
			t.Fatalf("missing %q in %s", want, html)
		}
	}
}
