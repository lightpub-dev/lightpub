package web

import (
	"fmt"
	"net/url"
)

func buildURLWithParams(url *url.URL, paramsToOverride map[string]string) string {
	q := url.Query()
	for k, v := range paramsToOverride {
		q.Set(k, v)
	}
	return fmt.Sprintf("%s?%s", url.Path, q.Encode())
}
