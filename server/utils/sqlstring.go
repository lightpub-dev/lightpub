package utils

import (
	"database/sql"

	"github.com/lightpub-dev/lightpub/config"
)

func ConvertSqlHost(host sql.NullString) string {
	if !host.Valid {
		return config.BaseURL
	}
	return host.String
}

func ConvertSqlStringToPtr(s sql.NullString) *string {
	if !s.Valid {
		return nil
	}
	result := s.String
	return &result

}
