package api

import "fmt"

func (h *Handler) AbsoluteURL(url string) string {
	return fmt.Sprintf("%s%s", h.BaseURL, url)
}
