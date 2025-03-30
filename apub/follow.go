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

type FollowActivity struct {
	ID     URI      `json:"id" validate:"required,http_url"`
	Kind   string   `json:"type" validate:"required,eq=Follow"`
	Actor  URI      `json:"actor" validate:"required,http_url"`
	Object ObjectID `json:"object" validate:"required"`
}

func (FollowActivity) InboxActivity() {}

func (FollowActivity) IDCheck() error {
	return nil
}

func NewFollowActivity(
	followURL URI,
	followerURL URI,
	followeeURL URI,
) FollowActivity {
	return FollowActivity{
		ID:     followURL,
		Kind:   "Follow",
		Actor:  followerURL,
		Object: NewObjectID(followeeURL),
	}
}

func (f FollowActivity) AsUndoable() UndoableActivity {
	return UndoableActivity{
		Kind:   UndoableActivityTypeFollow,
		Follow: &f,
	}
}

func (f FollowActivity) AsRejectable() RejectableActivity {
	return RejectableActivity{
		Kind:   RejectableActivityTypeFollow,
		Follow: &f,
	}
}

func (f FollowActivity) AsAcceptable() AcceptableActivity {
	return AcceptableActivity{
		Kind:   AcceptableActivityTypeFollow,
		Follow: &f,
	}
}
