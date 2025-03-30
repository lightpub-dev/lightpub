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

package apub

import "github.com/lightpub-dev/lightpub/types"

type AnnounceActivity struct {
	ID     string   `json:"id" validate:"required"`
	Kind   string   `json:"type" validate:"required"`
	Actor  URI      `json:"actor" validate:"required"`
	To     []string `json:"to"`
	Cc     []string `json:"cc"`
	Object ObjectID `json:"object" validate:"required"`
}

func (AnnounceActivity) InboxActivity() {}

func (a *AnnounceActivity) InferredVisibility() types.NoteVisibility {
	return inferVisibility(a.To, a.Cc)
}
