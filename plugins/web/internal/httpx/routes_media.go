package httpx

import (
	"fmt"
	"io"
	"net/http"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/session"
	"github.com/marqdo/marqdo/plugins/web/internal/storage"
	"github.com/marqdo/marqdo/plugins/web/internal/upload"
)

type uploadRoute struct {
	Field      string
	StorageURL string
	Prefix     string
	MaxBytes   uint64
	Types      any
}

type downloadRoute struct {
	StorageURL  string
	Disposition string
}

func parseUploadRoute(v any) uploadRoute {
	m, _ := v.(map[string]any)
	max := uint64(0)
	if n, ok := m["max_bytes"].(float64); ok {
		max = uint64(n)
	}
	return uploadRoute{
		Field:      strOpt(m, "field", "file"),
		StorageURL: strOpt(m, "storage_url", ""),
		Prefix:     strOpt(m, "prefix", "uploads/"),
		MaxBytes:   max,
		Types:      m["types"],
	}
}

func parseDownloadRoute(v any) downloadRoute {
	m, _ := v.(map[string]any)
	return downloadRoute{
		StorageURL:  strOpt(m, "storage_url", ""),
		Disposition: strOpt(m, "disposition", "attachment"),
	}
}

func (st *state) mountUploadRoutes(mux *http.ServeMux) {
	for _, path := range sortedKeys(st.uploadRoutes) {
		routePath := path
		cfg := parseUploadRoute(st.uploadRoutes[path])
		mux.HandleFunc("POST "+routePath, func(w http.ResponseWriter, r *http.Request) {
			st.handleUpload(w, r, routePath, cfg)
		})
	}
}

func (st *state) mountDownloadRoutes(mux *http.ServeMux) {
	for _, path := range sortedKeys(st.downloadRoutes) {
		routePath := path
		cfg := parseDownloadRoute(st.downloadRoutes[path])
		pattern := "GET " + goMuxPattern(routePath)
		mux.HandleFunc(pattern, func(w http.ResponseWriter, r *http.Request) {
			st.handleDownload(w, r, routePath, cfg)
		})
	}
}

func (st *state) mountGalleryRoutes(mux *http.ServeMux) {
	for _, path := range sortedKeys(st.galleryRoutes) {
		cfg, _ := st.galleryRoutes[path].(map[string]any)
		mux.HandleFunc("GET "+path, func(w http.ResponseWriter, r *http.Request) {
			st.writeGallery(w, cfg)
		})
	}
}

func (st *state) handleUpload(w http.ResponseWriter, r *http.Request, routePath string, cfg uploadRoute) {
	_, hadCookie := session.IDFromCookie(r.Header.Get("Cookie"))
	sid, _, setCookie := resolveSession(r)

	if err := r.ParseMultipartForm(32 << 20); err != nil {
		jsonErr(w, http.StatusBadRequest, fmt.Sprintf("multipart: %v", err))
		return
	}

	csrfToken := ""
	if vals := r.MultipartForm.Value["_csrf"]; len(vals) > 0 {
		csrfToken = vals[0]
	}

	var fileName, fileCT string
	fileCT = "application/octet-stream"
	var fileBytes []byte

	if files := r.MultipartForm.File[cfg.Field]; len(files) > 0 {
		fh := files[0]
		fileName = fh.Filename
		if fh.Header != nil {
			if ct := fh.Header.Get("Content-Type"); ct != "" {
				fileCT = ct
			}
		}
		f, err := fh.Open()
		if err != nil {
			jsonErr(w, http.StatusBadRequest, fmt.Sprintf("open file: %v", err))
			return
		}
		defer f.Close()
		b, err := io.ReadAll(f)
		if err != nil {
			jsonErr(w, http.StatusBadRequest, fmt.Sprintf("read file: %v", err))
			return
		}
		fileBytes = b
	} else {
		// Fallback: walk File map keys.
		for name, fhs := range r.MultipartForm.File {
			if name == cfg.Field && len(fhs) > 0 {
				fh := fhs[0]
				fileName = fh.Filename
				f, err := fh.Open()
				if err != nil {
					jsonErr(w, http.StatusBadRequest, fmt.Sprintf("open file: %v", err))
					return
				}
				defer f.Close()
				fileBytes, err = io.ReadAll(f)
				if err != nil {
					jsonErr(w, http.StatusBadRequest, fmt.Sprintf("read file: %v", err))
					return
				}
				break
			}
		}
	}

	if hadCookie && !session.ValidateCSRF(sid, csrfToken) {
		jsonErr(w, http.StatusForbidden, "Invalid or missing CSRF token")
		return
	}

	if len(fileBytes) == 0 {
		jsonErr(w, http.StatusBadRequest, "missing file field")
		return
	}

	size := uint64(len(fileBytes))
	check := upload.Validate(fileName, fileCT, size, cfg.MaxBytes, cfg.Types)
	if ok, _ := check["ok"].(bool); !ok {
		errMsg := strOpt(check, "error", "validation failed")
		jsonErr(w, http.StatusBadRequest, errMsg)
		return
	}
	ct := strOpt(check, "content_type", fileCT)

	key, err := upload.MakeKey(cfg.Prefix, fileName)
	if err != nil {
		jsonErr(w, http.StatusBadRequest, err.Error())
		return
	}
	saved, err := storage.PutBytes(cfg.StorageURL, key, fileBytes, ct)
	if err != nil {
		jsonErr(w, http.StatusInternalServerError, err.Error())
		return
	}

	body := map[string]any{
		"ok":           true,
		"key":          key,
		"size":         float64(size),
		"content_type": ct,
	}
	if sz, ok := saved["size"]; ok {
		body["size"] = sz
	}
	appendSetCookie(w, setCookie)
	writeJSON(w, http.StatusOK, body)
}

func (st *state) handleDownload(w http.ResponseWriter, r *http.Request, routePath string, cfg downloadRoute) {
	key := downloadKeyFromRequest(r, routePath)
	if key == "" {
		jsonErr(w, http.StatusBadRequest, "missing key")
		return
	}

	bytes, contentType, filename, found, err := storage.ReadBytes(cfg.StorageURL, key)
	if err != nil {
		jsonErr(w, http.StatusInternalServerError, err.Error())
		return
	}
	if !found {
		jsonErr(w, http.StatusNotFound, "not found")
		return
	}

	etag := weakETag(bytes)
	if inm := r.Header.Get("If-None-Match"); inm != "" {
		for _, part := range strings.Split(inm, ",") {
			if strings.TrimSpace(part) == etag {
				w.WriteHeader(http.StatusNotModified)
				return
			}
		}
	}

	disp := "attachment"
	if strings.EqualFold(cfg.Disposition, "inline") {
		disp = "inline"
	}
	safeName := strings.ReplaceAll(filename, `"`, "_")
	w.Header().Set("Content-Type", contentType)
	w.Header().Set("Content-Disposition", fmt.Sprintf("%s; filename=\"%s\"", disp, safeName))
	w.Header().Set("ETag", etag)
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(bytes)
}

func downloadKeyFromRequest(r *http.Request, routePath string) string {
	for _, name := range downloadParamNames(routePath) {
		if v := strings.TrimPrefix(r.PathValue(name), "/"); v != "" {
			return v
		}
	}
	if v := strings.TrimPrefix(r.PathValue("key"), "/"); v != "" {
		return v
	}
	return ""
}

func downloadParamNames(routePath string) []string {
	var names []string
	for _, seg := range strings.Split(routePath, "/") {
		if strings.HasPrefix(seg, "{*") && strings.HasSuffix(seg, "}") {
			names = append(names, strings.TrimSuffix(strings.TrimPrefix(seg, "{*"), "}"))
		} else if strings.HasPrefix(seg, "{") && strings.HasSuffix(seg, "}") {
			inner := strings.TrimSuffix(strings.TrimPrefix(seg, "{"), "}")
			if strings.HasSuffix(inner, "...") {
				inner = strings.TrimSuffix(inner, "...")
			}
			if inner != "" {
				names = append(names, inner)
			}
		}
	}
	return names
}

func (st *state) writeGallery(w http.ResponseWriter, cfg map[string]any) {
	storageURL := strOpt(cfg, "storage", "")
	prefix := strOpt(cfg, "prefix", "")
	title := strOpt(cfg, "title", "Gallery")
	downloadBase := strOpt(cfg, "download_base", "/_media")

	listed, err := storage.List(storageURL, prefix)
	keys := []any{}
	if err == nil {
		if k, ok := listed["keys"].([]string); ok {
			for _, s := range k {
				keys = append(keys, s)
			}
		} else if k, ok := listed["keys"].([]any); ok {
			keys = k
		}
	}

	var items strings.Builder
	for _, k := range keys {
		key, _ := k.(string)
		if key == "" {
			continue
		}
		href := downloadBase + "/" + strings.TrimPrefix(key, "/")
		name := key
		if i := strings.LastIndexByte(key, '/'); i >= 0 {
			name = key[i+1:]
		}
		lower := strings.ToLower(name)
		isImg := strings.HasSuffix(lower, ".png") || strings.HasSuffix(lower, ".jpg") ||
			strings.HasSuffix(lower, ".jpeg") || strings.HasSuffix(lower, ".gif") ||
			strings.HasSuffix(lower, ".webp") || strings.HasSuffix(lower, ".svg")
		if isImg {
			items.WriteString(fmt.Sprintf(
				"<figure class=\"gal-item\"><a href=\"%s\"><img src=\"%s\" alt=\"%s\"/></a><figcaption>%s</figcaption></figure>",
				esc(href), esc(href), esc(name), esc(name)))
		} else {
			items.WriteString(fmt.Sprintf(
				"<figure class=\"gal-item\"><a class=\"gal-file\" href=\"%s\">%s</a></figure>",
				esc(href), esc(name)))
		}
	}
	itemHTML := items.String()
	if itemHTML == "" {
		itemHTML = "<p class=\"gal-empty\">No media yet.</p>"
	}

	html := fmt.Sprintf(`<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>%s</title>
<style>
body{font-family:system-ui,sans-serif;margin:0;padding:1.5rem;background:#f6f4ef;color:#1a1a1a}
h1{margin:0 0 1rem;font-size:1.5rem}
.gal{display:grid;grid-template-columns:repeat(auto-fill,minmax(140px,1fr));gap:1rem}
.gal-item{margin:0;background:#fff;border:1px solid #ddd;border-radius:6px;overflow:hidden}
.gal-item img{display:block;width:100%%;height:120px;object-fit:cover}
.gal-item figcaption,.gal-file{display:block;padding:.5rem;font-size:.85rem;word-break:break-all}
.gal-empty{color:#666}
</style></head>
<body><h1>%s</h1><div class="gal">%s</div></body></html>`,
		esc(title), esc(title), itemHTML)

	etag := weakETag([]byte(html))
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.Header().Set("ETag", etag)
	w.WriteHeader(http.StatusOK)
	_, _ = io.WriteString(w, html)
}
