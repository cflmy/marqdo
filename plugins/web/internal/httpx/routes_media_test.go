package httpx

import (
	"bytes"
	"mime/multipart"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestHandleUploadRejectsOversizedBeforeFullBuffer(t *testing.T) {
	st := &state{
		uploadRoutes: map[string]any{
			"/upload": map[string]any{
				"field":       "file",
				"storage_url": "memory://t",
				"prefix":      "u",
				"max_bytes":   float64(64),
			},
		},
	}
	cfg := parseUploadRoute(st.uploadRoutes["/upload"])
	var body bytes.Buffer
	w := multipart.NewWriter(&body)
	part, err := w.CreateFormFile("file", "big.txt")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := part.Write([]byte(strings.Repeat("x", 200))); err != nil {
		t.Fatal(err)
	}
	_ = w.Close()

	req := httptest.NewRequest(http.MethodPost, "/upload", &body)
	req.Header.Set("Content-Type", w.FormDataContentType())
	rr := httptest.NewRecorder()
	st.handleUpload(rr, req, "/upload", cfg)
	if rr.Code == http.StatusOK {
		t.Fatalf("expected reject for oversized upload, got %d body=%s", rr.Code, rr.Body.String())
	}
	if rr.Code != http.StatusRequestEntityTooLarge && rr.Code != http.StatusBadRequest {
		t.Fatalf("expected 413 or 400, got %d body=%s", rr.Code, rr.Body.String())
	}
}
