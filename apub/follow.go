package apub

type FollowActivity struct {
	ID     URI      `json:"id" validate:"required,http_url"`
	Kind   string   `json:"type" validate:"required,eq=Follow"`
	Actor  URI      `json:"actor" validate:"required,http_url"`
	Object ObjectID `json:"object" validate:"required"`
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
		Kind:         UndoableActivityTypeFollow,
		FollowObject: &f,
	}
}
