package service

import (
	"errors"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/types"
	"gorm.io/gorm"
)

func (s *ServiceState) FindUserByIDRaw(id types.UserID) (*db.User, error) {
	var user db.User
	if err := s.DB().Where("id = ?", id).First(&user).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}

	return &user, nil
}

func (s *ServiceState) FindUserByID(id types.UserID) (*types.SimpleUser, error) {
	user, err := s.FindUserByIDRaw(id)
	if err != nil {
		return nil, err
	}
	if user == nil {
		return nil, nil
	}

	return &types.SimpleUser{
		ID:       user.ID,
		Username: user.Username,
		Domain:   user.Domain,
		Nickname: user.Nickname,
		Bio:      user.Bio,
		Avatar:   user.Avatar,
	}, nil
}
