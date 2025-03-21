package kv

import (
	"context"
	"io"
	"time"

	"github.com/lightpub-dev/lightpub/failure"
)

var (
	ErrKvMiss = failure.NewError(404, "key not found")
)

const (
	TtlNoExpiration = time.Duration(0)
)

type Cache interface {
	Get(ctx context.Context, key string) (io.Reader, error)
	Set(ctx context.Context, key string, value io.Reader, ttl time.Duration) error
	Del(ctx context.Context, key string) error
}
