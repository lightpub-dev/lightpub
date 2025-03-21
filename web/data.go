package web

import (
	"io"
	"os"

	"github.com/go-playground/validator/v10"
	"github.com/lightpub-dev/lightpub/auth"
	"github.com/lightpub-dev/lightpub/service"
	"gopkg.in/yaml.v3"
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
	Service          service.Config `yaml:"service"`
	Auth             auth.Config    `yaml:"auth"`
	RegistrationOpen bool           `yaml:"registration_open"`
}

func NewStateFromConfigFile(path string) (*State, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	fBytes, err := io.ReadAll(f)
	if err != nil {
		return nil, err
	}

	var config Config
	if err := yaml.Unmarshal(fBytes, &config); err != nil {
		return nil, err
	}

	return NewState(config), nil
}

func NewState(config Config) *State {
	return &State{
		service:          service.NewStateFromConfig(config.Service),
		auth:             auth.NewStateFromConfig(config.Auth),
		registrationOpen: config.RegistrationOpen,
	}
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
