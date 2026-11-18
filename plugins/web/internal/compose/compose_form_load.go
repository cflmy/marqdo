package compose

import (
	"fmt"
	"strings"
)

// ComposeFormLoad stamps page.form_load so render prefills the composed form
// from DB by route/query id (desk edit pages without author JS).
func ComposeFormLoad(page any, table, idParam string) (any, error) {
	obj := asObject(page)
	table = strings.TrimSpace(table)
	if table == "" {
		return nil, fmt.Errorf("compose_form_load requires non-empty table")
	}
	spec := map[string]any{"table": table}
	if s := strings.TrimSpace(idParam); s != "" {
		spec["id_param"] = s
	} else {
		spec["id_param"] = "id"
	}
	obj["form_load"] = spec
	return obj, nil
}
