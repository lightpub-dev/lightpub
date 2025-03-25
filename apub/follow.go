package apub

type FollowActivity struct {
	ID     string   `json:"id" validate:"required"`
	Kind   string   `json:"type" validate:"required"`
	Actor  URI      `json:"actor" validate:"required"`
	Object ObjectID `json:"object" validate:"required"`
}
