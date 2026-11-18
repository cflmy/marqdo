package compose

import (
	"fmt"
	"strings"
)

// ComposeAuthForm stamps page.auth_form for progressive-enhancement login/register.
// action is the form POST path (e.g. /login, /desk/login, /register).
func ComposeAuthForm(page any, action, submit, formID, errID, target, nextDefault, kind string) (any, error) {
	action = strings.TrimSpace(action)
	if action == "" {
		return nil, fmt.Errorf("compose_auth_form requires non-empty action")
	}
	obj := asObject(page)
	kind = strings.TrimSpace(strings.ToLower(kind))
	if kind == "" {
		if strings.Contains(strings.ToLower(action), "register") {
			kind = "register"
		} else {
			kind = "login"
		}
	}
	spec := map[string]any{
		"action": action,
		"kind":   kind,
	}
	if s := strings.TrimSpace(submit); s != "" {
		spec["submit"] = s
	}
	if s := strings.TrimSpace(formID); s != "" {
		spec["form_id"] = s
	}
	if s := strings.TrimSpace(errID); s != "" {
		spec["err_id"] = s
	}
	if s := strings.TrimSpace(target); s != "" {
		spec["target"] = s
	}
	if s := strings.TrimSpace(nextDefault); s != "" {
		spec["next"] = s
	}
	obj["auth_form"] = spec
	return obj, nil
}
