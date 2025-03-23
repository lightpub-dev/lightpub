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

package web

import (
	"io"
	"net/url"
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
	hxRedirect      = "HX-Redirect"
	hxTrigger       = "HX-Trigger"
	hxRefresh       = "HX-Refresh"
	trueHeaderValue = "true"
	cacheControl    = "Cache-Control"
	vary            = "Vary"
	acceptLanguage  = "Accept-Language"

	paginationSize   = 20
	paginationSizeP1 = paginationSize + 1
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

func (s *State) BaseURL() *url.URL {
	return s.service.BaseURL()
}

func (s *State) MyDomain() string {
	return s.service.MyDomain()
}
