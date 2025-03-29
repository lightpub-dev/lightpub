package apub

import (
	"encoding/json"
	"fmt"
)

type UndoableActivityType string

const (
	UndoableActivityTypeFollow UndoableActivityType = "Follow"
)

type UndoActivity struct {
	ID     string           `json:"id" validate:"required,http_url"`
	Kind   string           `json:"type" validate:"required,eq=Undo"`
	Actor  URI              `json:"actor" validate:"required,http_url"`
	Object UndoableActivity `json:"object" validate:"required"`
}

func NewUndoActivity(
	undoer URI,
	undoableActivity UndoableActivity,
) UndoActivity {
	undoID := activityIDFromObject("Undo", undoableActivity.ID())
	return UndoActivity{
		ID:     undoID,
		Kind:   "Undo",
		Actor:  undoer,
		Object: undoableActivity,
	}
}

type UndoableActivity struct {
	Kind UndoableActivityType `validate:"required,oneof=Follow"`

	FollowObject *FollowActivity
}

func (u UndoableActivity) ID() string {
	switch u.Kind {
	case UndoableActivityTypeFollow:
		return u.FollowObject.ID
	}

	panic("unknown undoable object type")
}

func (u UndoableActivity) MarshalJSON() ([]byte, error) {
	switch u.Kind {
	case UndoableActivityTypeFollow:
		return json.Marshal(*u.FollowObject)
	}

	return nil, fmt.Errorf("unknown undoable object type: %s", u.Kind)
}

func (u *UndoableActivity) UnmarshalJSON(data []byte) error {
	_, typ, err := unmarshalToMapAndType(data)
	if err != nil {
		return err
	}

	switch typ {
	case "Follow":
		u.Kind = UndoableActivityTypeFollow
		var f FollowActivity
		if err := json.Unmarshal(data, &f); err != nil {
			return fmt.Errorf("error unmarshalling follow object: %w", err)
		}
		if err := validate.Struct(f); err != nil {
			return fmt.Errorf("error validating follow object: %w", err)
		}
		u.FollowObject = &f
	}

	return fmt.Errorf("unknown undoable object type: %s", typ)
}
