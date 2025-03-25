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
	ID     string           `json:"id" validate:"required"`
	Kind   string           `json:"type" validate:"required"`
	Actor  URI              `json:"actor" validate:"required"`
	Object UndoableActivity `json:"object" validate:"required"`
}

type UndoableActivity struct {
	Kind UndoableActivityType

	FollowObject *FollowActivity
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
