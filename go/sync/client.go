package mpsync

import (
	"fmt"
	"net/http"
	"net/url"
)

// MockableClient - HTTP interface to metaparticle-sync sidecar container.
type MockableClient interface {
	Get(url.URL) (int, error)
	Put(url.URL) (int, error)
}

type httpClient struct {
	http.Client
}

func (h httpClient) Get(url url.URL) (int, error) {
	req, _ := http.NewRequest("GET", url.String(), nil)

	resp, err := h.Do(req)
	if err != nil {
		return -1, err
	}

	return resp.StatusCode, nil
}

func (h httpClient) Put(url url.URL) (int, error) {
	req, _ := http.NewRequest("PUT", url.String(), nil)

	resp, err := h.Do(req)
	if err != nil {
		return -1, err
	}

	if resp.StatusCode == 200 || resp.StatusCode == 409 {
		return resp.StatusCode, nil
	}

	return -1, fmt.Errorf("Unexpected status: %d", resp.StatusCode)
}
