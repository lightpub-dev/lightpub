package web

import (
	"github.com/go-playground/validator/v10"
	"github.com/lightpub-dev/lightpub/auth"
	"github.com/lightpub-dev/lightpub/service"
)

var (
	validate = validator.New(validator.WithRequiredStructEnabled())
)

const (
	hxRedirect = "HX-Redirect"
)

type State struct {
	service *service.State
	auth    *auth.State

	registrationOpen bool
}

func (s *State) Service() *service.State {
	return s.service
}
