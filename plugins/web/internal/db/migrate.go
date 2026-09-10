package db

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

// Migrate applies versioned SQL steps (SQLite only).
// steps: list of {version|版本|ver, sql|SQL} maps (GFM columnar via AsRows).
// Returns {"ok":true,"from":N,"to":M,"applied":[…]}.
func Migrate(url string, steps any) (map[string]any, error) {
	lower := strings.ToLower(ResolveURL(url))
	if strings.HasPrefix(lower, "postgres://") || strings.HasPrefix(lower, "postgresql://") {
		return nil, fmt.Errorf("db.migrate is SQLite-only in this wave (use postgres migration tooling)")
	}

	normalized := table.AsRows(steps)
	arr, ok := normalized.([]any)
	if !ok {
		return nil, fmt.Errorf("migrate steps must be a list")
	}

	type step struct {
		version int64
		sql     string
	}
	var parsed []step
	for _, row := range arr {
		// Bare SQL string → treat as sequential versions later if needed;
		// Rust requires maps; also accept plain SQL strings with auto version.
		if s, ok := row.(string); ok {
			s = strings.TrimSpace(s)
			if s == "" {
				return nil, fmt.Errorf("migrate step has empty SQL")
			}
			parsed = append(parsed, step{version: 0, sql: s}) // version filled below
			continue
		}
		obj, ok := row.(map[string]any)
		if !ok {
			return nil, fmt.Errorf("migrate step must be a map")
		}
		var ver any
		for _, k := range []string{"version", "版本", "ver"} {
			if v, ok := obj[k]; ok {
				ver = v
				break
			}
		}
		if ver == nil {
			return nil, fmt.Errorf("migrate step missing `version`/`版本`")
		}
		version, err := migrateVersion(ver)
		if err != nil {
			return nil, err
		}
		if version <= 0 {
			return nil, fmt.Errorf("migrate version must be positive, got %d", version)
		}
		sqlStmt := ""
		for _, k := range []string{"sql", "SQL", "Sql"} {
			if v, ok := obj[k]; ok {
				sqlStmt = cellStr(v)
				break
			}
		}
		if strings.TrimSpace(sqlStmt) == "" {
			return nil, fmt.Errorf("migrate version %d has empty SQL", version)
		}
		parsed = append(parsed, step{version: version, sql: sqlStmt})
	}

	// Auto-number bare SQL strings (1-based in list order among string-only steps,
	// or continue after max explicit version when mixed — prefer Rust map path).
	var auto int64
	for i := range parsed {
		if parsed[i].version == 0 {
			auto++
			parsed[i].version = auto
		} else if parsed[i].version > auto {
			auto = parsed[i].version
		}
	}

	// Sort by version (stable insertion sort).
	for i := 1; i < len(parsed); i++ {
		j := i
		for j > 0 && parsed[j-1].version > parsed[j].version {
			parsed[j-1], parsed[j] = parsed[j], parsed[j-1]
			j--
		}
	}
	for i := 1; i < len(parsed); i++ {
		if parsed[i-1].version == parsed[i].version {
			return nil, fmt.Errorf("duplicate migrate version %d", parsed[i].version)
		}
	}

	db, err := open(url)
	if err != nil {
		return nil, err
	}
	if _, err := db.Exec(`CREATE TABLE IF NOT EXISTS "_marqdo_migrations" (
		version INTEGER PRIMARY KEY,
		applied_at TEXT NOT NULL
	)`); err != nil {
		return nil, err
	}
	var current int64
	if err := db.QueryRow(`SELECT COALESCE(MAX(version), 0) FROM "_marqdo_migrations"`).Scan(&current); err != nil {
		current = 0
	}

	var applied []any
	for _, st := range parsed {
		if st.version <= current {
			continue
		}
		tx, err := db.Begin()
		if err != nil {
			return nil, err
		}
		if _, err := tx.Exec(st.sql); err != nil {
			_ = tx.Rollback()
			return nil, fmt.Errorf("migrate version %d failed: %w", st.version, err)
		}
		if _, err := tx.Exec(
			`INSERT INTO "_marqdo_migrations" (version, applied_at) VALUES (?, datetime('now'))`,
			st.version,
		); err != nil {
			_ = tx.Rollback()
			return nil, err
		}
		if err := tx.Commit(); err != nil {
			return nil, err
		}
		applied = append(applied, float64(st.version))
	}
	if applied == nil {
		applied = []any{}
	}
	to := current
	if len(applied) > 0 {
		to = int64(applied[len(applied)-1].(float64))
	}
	return map[string]any{
		"ok":      true,
		"from":    float64(current),
		"to":      float64(to),
		"applied": applied,
	}, nil
}

func migrateVersion(ver any) (int64, error) {
	switch t := ver.(type) {
	case float64:
		return int64(t), nil
	case int64:
		return t, nil
	case int:
		return int64(t), nil
	case string:
		n, err := strconv.ParseInt(strings.TrimSpace(t), 10, 64)
		if err != nil {
			return 0, fmt.Errorf("bad migrate version `%s`", t)
		}
		return n, nil
	default:
		return 0, fmt.Errorf("migrate version must be an integer")
	}
}
