package render

import (
	"fmt"
	"regexp"
	"strings"
)

var (
	rePostForm = regexp.MustCompile(`(?is)<form\b([^>]*\bmethod\s*=\s*["']post["'][^>]*)>([\s\S]*?)</form>`)
	reAction   = regexp.MustCompile(`(?i)\baction\s*=\s*["']([^"']*)["']`)
	reCSRFIn   = regexp.MustCompile(`(?i)\bname\s*=\s*["']_csrf["']`)
	reNextIn   = regexp.MustCompile(`(?i)\bname\s*=\s*["']next["']`)
)

// AuthFormSpec describes a progressive-enhancement login/register form.
type AuthFormSpec struct {
	Action      string
	Submit      string
	FormID      string
	ErrID       string
	Target      string
	NextDefault string
	Kind        string // login | register
}

// AuthFormFromPage reads page["auth_form"].
func AuthFormFromPage(page map[string]any) *AuthFormSpec {
	if page == nil {
		return nil
	}
	raw, ok := page["auth_form"].(map[string]any)
	if !ok || raw == nil {
		return nil
	}
	spec := &AuthFormSpec{
		Action:      strings.TrimSpace(text(raw["action"])),
		Submit:      strings.TrimSpace(text(raw["submit"])),
		FormID:      strings.TrimSpace(text(raw["form_id"])),
		ErrID:       strings.TrimSpace(text(raw["err_id"])),
		Target:      strings.TrimSpace(text(raw["target"])),
		NextDefault: strings.TrimSpace(text(raw["next"])),
		Kind:        strings.TrimSpace(strings.ToLower(text(raw["kind"]))),
	}
	if spec.Action == "" {
		return nil
	}
	if spec.Submit == "" {
		if spec.Kind == "register" {
			spec.Submit = "注册"
		} else {
			spec.Submit = "登录"
		}
	}
	if spec.FormID == "" {
		if spec.Kind == "register" {
			spec.FormID = "site-register-form"
		} else {
			spec.FormID = "site-login-form"
		}
	}
	if spec.ErrID == "" {
		spec.ErrID = "site-auth-err"
	}
	if spec.Kind == "" {
		if strings.Contains(spec.Action, "register") {
			spec.Kind = "register"
		} else {
			spec.Kind = "login"
		}
	}
	return spec
}

// RenderAuthFormHTML builds a complete auth form (CSRF/next filled at inject time too).
func RenderAuthFormHTML(spec *AuthFormSpec, csrf, next, flash string) string {
	if spec == nil {
		return ""
	}
	next = strings.TrimSpace(next)
	if next == "" {
		next = strings.TrimSpace(spec.NextDefault)
	}
	var b strings.Builder
	fmt.Fprintf(&b, `<form id="%s" class="auth-form mq-auth-form" method="post" action="%s">`, esc(spec.FormID), esc(spec.Action))
	if csrf != "" {
		fmt.Fprintf(&b, `<input type="hidden" name="_csrf" value="%s"/>`, esc(csrf))
	}
	if next != "" && spec.Kind != "register" {
		fmt.Fprintf(&b, `<input type="hidden" name="next" value="%s"/>`, esc(next))
	}
	b.WriteString(`<label>用户名<input name="username" autocomplete="username" required autofocus/></label>`)
	b.WriteString(`<label>密码<input name="password" type="password" autocomplete="`)
	if spec.Kind == "register" {
		b.WriteString(`new-password`)
	} else {
		b.WriteString(`current-password`)
	}
	b.WriteString(`" required`)
	if spec.Kind == "register" {
		b.WriteString(` minlength="4"`)
	}
	b.WriteString(`/></label>`)
	fmt.Fprintf(&b, `<button type="submit">%s</button>`, esc(spec.Submit))
	b.WriteString(`</form>`)
	flashAttr := ` hidden`
	flashText := ""
	if strings.TrimSpace(flash) != "" {
		flashAttr = ""
		flashText = strings.TrimSpace(flash)
	}
	fmt.Fprintf(&b, `<p id="%s" class="auth-err"%s>%s</p>`, esc(spec.ErrID), flashAttr, esc(flashText))
	return b.String()
}

// ApplyAuthFormIntoIntro injects generated form into intro HTML target, or appends it.
func ApplyAuthFormIntoIntro(intro string, spec *AuthFormSpec, csrf, next, flash string) string {
	if spec == nil {
		return intro
	}
	formHTML := RenderAuthFormHTML(spec, csrf, next, flash)
	target := strings.TrimSpace(spec.Target)
	if target != "" {
		if injected, ok := injectHTMLIntoID(intro, target, formHTML); ok {
			return injected
		}
	}
	if intro == "" {
		return formHTML
	}
	return intro + formHTML
}

// IsAuthAction reports whether a form action path is a login/register endpoint.
func IsAuthAction(action string) bool {
	a := strings.TrimSpace(action)
	if a == "" {
		return false
	}
	// Absolute same-origin paths only.
	if strings.HasPrefix(a, "http://") || strings.HasPrefix(a, "https://") || strings.HasPrefix(a, "//") {
		return false
	}
	path := a
	if i := strings.IndexByte(path, '?'); i >= 0 {
		path = path[:i]
	}
	path = strings.TrimRight(path, "/")
	if path == "" {
		path = "/"
	}
	base := path
	if i := strings.LastIndexByte(path, '/'); i >= 0 {
		base = path[i+1:]
	}
	return base == "login" || base == "register" ||
		strings.HasSuffix(path, "/login") || strings.HasSuffix(path, "/register")
}

// InjectAuthFields ensures POST auth forms carry _csrf / next, and surfaces flash errors.
func InjectAuthFields(html, csrf, next, flash string) string {
	csrf = strings.TrimSpace(csrf)
	next = strings.TrimSpace(next)
	flash = strings.TrimSpace(flash)
	if csrf == "" && next == "" && flash == "" {
		return html
	}
	out := rePostForm.ReplaceAllStringFunc(html, func(full string) string {
		m := rePostForm.FindStringSubmatch(full)
		if len(m) < 3 {
			return full
		}
		attrs, body := m[1], m[2]
		am := reAction.FindStringSubmatch(attrs)
		action := ""
		if len(am) > 1 {
			action = am[1]
		}
		if !IsAuthAction(action) {
			return full
		}
		var inject strings.Builder
		if csrf != "" && !reCSRFIn.MatchString(body) {
			fmt.Fprintf(&inject, `<input type="hidden" name="_csrf" value="%s"/>`, esc(csrf))
		}
		if next != "" && !reNextIn.MatchString(body) && !strings.Contains(strings.ToLower(action), "register") {
			fmt.Fprintf(&inject, `<input type="hidden" name="next" value="%s"/>`, esc(next))
		}
		if inject.Len() == 0 {
			return full
		}
		return `<form` + attrs + `>` + inject.String() + body + `</form>`
	})
	if flash != "" {
		out = surfaceAuthFlash(out, flash)
	}
	return out
}

var reErrSlot = regexp.MustCompile(`(?is)(<(?:p|div)\b[^>]*\bid\s*=\s*["'](?:site-auth-err|desk-login-err|auth-err)["'][^>]*)(>)([^<]*)(</(?:p|div)>)`)

func surfaceAuthFlash(html, flash string) string {
	replaced := false
	out := reErrSlot.ReplaceAllStringFunc(html, func(full string) string {
		m := reErrSlot.FindStringSubmatch(full)
		if len(m) < 5 {
			return full
		}
		replaced = true
		open := m[1]
		open = regexp.MustCompile(`(?i)\s*\bhidden\b`).ReplaceAllString(open, "")
		return open + m[2] + esc(flash) + m[4]
	})
	if replaced {
		return out
	}
	flashHTML := `<p class="flash err auth-err">` + esc(flash) + `</p>`
	if i := strings.Index(out, `<main`); i >= 0 {
		if j := strings.Index(out[i:], `>`); j >= 0 {
			at := i + j + 1
			return out[:at] + flashHTML + out[at:]
		}
	}
	return flashHTML + out
}
