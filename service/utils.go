package service

import (
	"database/sql"
	"time"
)

func stringPtrToSql(s *string) sql.NullString {
	if s == nil {
		return sql.NullString{}
	}
	return sql.NullString{String: *s, Valid: true}
}

func stringToSql(s string) sql.NullString {
	return sql.NullString{String: s, Valid: true}
}

func timePtrToSql(t *time.Time) sql.NullTime {
	if t == nil {
		return sql.NullTime{}
	}
	return sql.NullTime{Time: *t, Valid: true}
}

func sqlToTimePtr(t sql.NullTime) *time.Time {
	if !t.Valid {
		return nil
	}
	return &t.Time
}
