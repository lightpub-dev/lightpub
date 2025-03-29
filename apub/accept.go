package apub

import (
	"encoding/json"
	"fmt"
)

type AcceptableActivityType string

const (
	AcceptableActivityTypeFollow AcceptableActivityType = "Follow"
)

type AcceptActivity struct {
	ID     string             `json:"id" validate:"required,http_url"`
	Kind   string             `json:"type" validate:"required,eq=Accept"`
	Actor  URI                `json:"actor" validate:"required,http_url"`
	Object AcceptableActivity `json:"object" validate:"required"`
}

func NewAcceptActivityWithID(
	acceptID string,
	accepter URI,
	acceptableObject AcceptableActivity,
) AcceptActivity {
	return AcceptActivity{
		ID:     acceptID,
		Kind:   "Accept",
		Actor:  accepter,
		Object: acceptableObject,
	}
}

func NewAcceptActivity(
	accepter URI,
	acceptableObject AcceptableActivity,
) AcceptActivity {
	acceptID := activityIDFromObject("Accept", acceptableObject.ID())
	return NewAcceptActivityWithID(
		acceptID,
		accepter,
		acceptableObject,
	)
}

type AcceptableActivity struct {
	Kind AcceptableActivityType `validate:"required,oneof=Follow"`

	Follow *FollowActivity
}

func (a AcceptableActivity) ID() string {
	switch a.Kind {
	case AcceptableActivityTypeFollow:
		return a.Follow.ID
	}

	panic("unknown acceptable object type")
}

func (a AcceptableActivity) MarshalJSON() ([]byte, error) {
	switch a.Kind {
	case AcceptableActivityTypeFollow:
		return json.Marshal(*a.Follow)
	}

	return nil, fmt.Errorf("unknown acceptable object type: %s", a.Kind)
}

func (a *AcceptableActivity) UnmarshalJSON(data []byte) error {
	_, typ, err := unmarshalToMapAndType(data)
	if err != nil {
		return err
	}

	switch typ {
	case "Follow":
		a.Kind = AcceptableActivityTypeFollow
		var f FollowActivity
		if err := json.Unmarshal(data, &f); err != nil {
			return fmt.Errorf("error unmarshalling follow object: %w", err)
		}
		if err := validate.Struct(f); err != nil {
			return fmt.Errorf("error validating follow object: %w", err)
		}
		a.Follow = &f
		return nil
	}

	return fmt.Errorf("unknown acceptable object type: %s", typ)
}
