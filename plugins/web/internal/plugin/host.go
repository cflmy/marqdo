package plugin

import "encoding/json"

// HostFns holds callbacks into the Marqdo host (valid after marqdo_plugin_init).
type HostFns struct {
	Query func(name, argsJSON string) (string, error)
}

var host HostFns

// SetHost stores host callbacks from ABI init.
func SetHost(h HostFns) { host = h }

// Shutdown clears plugin global state (HTTP servers, pools, …).
func Shutdown() {
	host = HostFns{}
}

// Query runs an allowlisted host_query.
func Query(name, argsJSON string) (string, error) {
	if host.Query == nil {
		return "", errString("host_query not available")
	}
	return host.Query(name, argsJSON)
}

type errString string

func (e errString) Error() string { return string(e) }

// CallLibPath invokes `lib.member` on the site entry module (optional named args).
func CallLibPath(path string, args map[string]any) (any, error) {
	payload := map[string]any{"path": path}
	if len(args) > 0 {
		payload["args"] = args
	}
	raw, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}
	out, err := Query("call_lib_path", string(raw))
	if err != nil {
		return nil, err
	}
	var v any
	if err := json.Unmarshal([]byte(out), &v); err != nil {
		return nil, err
	}
	return v, nil
}
