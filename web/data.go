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

type Config struct {
	RegistrationOpen bool `json:"registration_open"`
}

func NewState(config Config) *State {

}

func (s *State) Service() *service.State {
	return s.service
}

func (s *State) Auth() *auth.State {
	return s.auth
}

func (s *State) DevMode() bool {
	return s.service.DevMode()
}
