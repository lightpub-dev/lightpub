/*
Lightpub: An activitypub server
Copyright (C) 2025 tinaxd

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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
