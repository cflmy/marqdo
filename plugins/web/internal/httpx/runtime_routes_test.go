package httpx

import (
	"testing"
)

func TestSystemRuntimeRoutesListed(t *testing.T) {
	// Compile-time presence: handlers must be addressable.
	var st state
	_ = st.handleRuntimeAdmit
	_ = st.handleRuntimeCancel
	_ = st.handleRuntimeEvents
	_ = st.handleRuntimeGet
}
