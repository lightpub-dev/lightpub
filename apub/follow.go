package apub

type FollowActivity struct {
	ID     URI      `json:"id" validate:"required,http_url"`
	Kind   string   `json:"type" validate:"required"`
	Actor  URI      `json:"actor" validate:"required,http_url"`
	Object ObjectID `json:"object" validate:"required"`
}
