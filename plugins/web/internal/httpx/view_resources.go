package httpx

import (
	"fmt"
	"net/http"
	"regexp"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/artifact"
	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/form"
	"github.com/marqdo/marqdo/plugins/web/internal/render"
	"github.com/marqdo/marqdo/plugins/web/internal/tenant"
)

// ValidateViewResource validates the closed Resource contracts without IO.
// Assembly calls this before listening; request rendering validates again.
func ValidateViewResource(resource map[string]any) (string, error) {
	kind, _ := resource["_type"].(string)
	switch kind {
	case "document_head":
		_, err := artifact.ParseDocumentHead(resource)
		return "", err
	case "data_query":
		if source, ok := resource["source"].(string); !ok || strings.TrimSpace(source) == "" {
			return "", fmt.Errorf("data_query: source must be a nonempty table name")
		}
		for _, key := range []string{"order", "link_prefix"} {
			if value, exists := resource[key]; exists {
				if _, ok := value.(string); !ok {
					return "", fmt.Errorf("data_query: %s must be text", key)
				}
			}
		}
		for _, key := range []string{"detail", "tenant_scope"} {
			if value, exists := resource[key]; exists {
				if _, ok := value.(bool); !ok {
					return "", fmt.Errorf("data_query: %s must be boolean", key)
				}
			}
		}
		if raw, exists := resource["limit"]; exists {
			n, ok := raw.(float64)
			if !ok || n < 1 || n > 1000 || n != float64(int64(n)) {
				return "", fmt.Errorf("data_query: limit must be an integer in 1..1000")
			}
		}
		if prefix, _ := resource["link_prefix"].(string); prefix != "" && (!strings.HasPrefix(prefix, "/") || strings.HasPrefix(prefix, "//")) {
			return "", fmt.Errorf("data_query: link_prefix must be site-relative")
		}
		if where := resource["where"]; where != nil {
			switch where.(type) {
			case map[string]any, []any:
			default:
				return "", fmt.Errorf("data_query: where must be a native condition map or list")
			}
		}
	case "form_ref":
		if id, ok := resource["id"].(string); !ok || strings.TrimSpace(id) == "" {
			return "", fmt.Errorf("form_ref: id must name an explicitly mounted form")
		}
	case "view_navigation":
		region, _ := resource["region"].(string)
		if region != "nav" && region != "sidebar" && region != "footer" {
			return "", fmt.Errorf("view_navigation: region must be nav, sidebar or footer")
		}
		switch resource["items"].(type) {
		case map[string]any, []any:
		default:
			return "", fmt.Errorf("view_navigation: items must be link records")
		}
	default:
		return "", fmt.Errorf("unsupported View Resource %q", kind)
	}
	return "", nil
}

var viewParameter = regexp.MustCompile(`\{([^{}]+)\}`)

// Copy parameterized conditions without mutating the retained startup nodes.
func resourceWhere(value any, params map[string]any) (any, error) {
	switch value := value.(type) {
	case string:
		var missing string
		out := viewParameter.ReplaceAllStringFunc(value, func(token string) string {
			key := token[1 : len(token)-1]
			actual, ok := params[key]
			if !ok {
				missing = key
				return token
			}
			return fmt.Sprint(actual)
		})
		if missing != "" {
			return nil, fmt.Errorf("data_query: missing route parameter %q", missing)
		}
		return out, nil
	case map[string]any:
		out := make(map[string]any, len(value))
		for key, item := range value {
			resolved, err := resourceWhere(item, params)
			if err != nil {
				return nil, err
			}
			out[key] = resolved
		}
		return out, nil
	case []any:
		out := make([]any, len(value))
		for i, item := range value {
			resolved, err := resourceWhere(item, params)
			if err != nil {
				return nil, err
			}
			out[i] = resolved
		}
		return out, nil
	default:
		return value, nil
	}
}

func (st *state) renderViewResource(resource, page map[string]any, request *http.Request) (string, error) {
	if _, err := ValidateViewResource(resource); err != nil {
		return "", err
	}
	switch resource["_type"] {
	case "document_head":
		return "", nil
	case "data_query":
		if st.dbURL == "" {
			return "", fmt.Errorf("data_query: the application has no database resource")
		}
		params, _ := page["params"].(map[string]any)
		where, err := resourceWhere(resource["where"], params)
		if err != nil {
			return "", err
		}
		scoped, _ := resource["tenant_scope"].(bool)
		if scoped && !st.tenant.Enabled() {
			return "", fmt.Errorf("data_query: scoped query requires a tenant policy")
		}
		if scoped || st.tenant.DefaultScope {
			id := tenant.FromContext(request.Context())
			if id == "" {
				return "", fmt.Errorf("data_query: tenant required")
			}
			where = tenant.MergeWhere(where, st.tenant.Column, id)
		}
		limit := int64(50)
		if n, ok := resource["limit"].(float64); ok {
			limit = int64(n)
		}
		order, _ := resource["order"].(string)
		rows, err := db.Select(st.dbURL, resource["source"].(string), limit, db.SelectOpts{Where: where, Order: order})
		if err != nil {
			return "", err
		}
		detail, _ := resource["detail"].(bool)
		prefix, _ := resource["link_prefix"].(string)
		return render.RenderQueryRows(rows["rows"], detail, prefix), nil
	case "form_ref":
		id := resource["id"].(string)
		selected, ok := st.forms[id].(map[string]any)
		if !ok {
			return "", fmt.Errorf("form_ref: form %q was not explicitly mounted", id)
		}
		csrf, _ := page["_csrf"].(string)
		return form.RenderBodyCtx(selected, id, nil, nil, csrf, st.requestContext(request, nil)), nil
	case "view_navigation":
		region := resource["region"].(string)
		navigation := map[string]any{region: resource["items"]}
		if region == "nav" {
			navigation["nav"] = resource["items"]
		}
		if logged, ok := page["_logged_in"]; ok {
			navigation["_logged_in"] = logged
		}
		if user, ok := page["_nav_user"]; ok {
			navigation["_nav_user"] = user
		}
		return render.RenderPage(navigation, "", region), nil
	}
	return "", fmt.Errorf("unsupported View Resource")
}
