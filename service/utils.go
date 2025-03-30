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

func stringToNullableSql(s string) sql.NullString {
	if s == "" {
		return sql.NullString{}
	}
	return sql.NullString{String: s, Valid: true}
}

func timePtrToSql(t *time.Time) sql.NullTime {
	if t == nil {
		return sql.NullTime{}
	}
	return sql.NullTime{Time: *t, Valid: true}
}

func sqlToStringPtr(t sql.NullString) *string {
	if !t.Valid {
		return nil
	}
	return &t.String
}

func sqlToString(t sql.NullString) string {
	return t.String
}

func sqlToTimePtr(t sql.NullTime) *time.Time {
	if !t.Valid {
		return nil
	}
	return &t.Time
}
