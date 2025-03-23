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
	"errors"
	"net/http"
	"strconv"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/lightpub-dev/lightpub/failure"
	"github.com/lightpub-dev/lightpub/service"
	"github.com/lightpub-dev/lightpub/types"
)

const (
	jwtCookieAge = 24 * 60 * 60 // 24 hours
)

func (s *State) RegisterUser(c echo.Context) error {
	if !s.registrationOpen {
		return failure.NewError(http.StatusBadRequest, "registration is closed")
	}

	var req struct {
		Username string `json:"username" validate:"required,min=3,max=64"`
		Nickname string `json:"nickname" validate:"required,min=1,max=128"`
		Password string `json:"password" validate:"required,min=8,max=64"`
	}
	if err := c.Bind(&req); err != nil {
		return errBadInput
	}
	if err := validate.Struct(&req); err != nil {
		return err
	}

	userID, err := s.service.CreateNewLocalUser(c.Request().Context(), service.UserCreateParams{
		Username: req.Username,
		Nickname: req.Nickname,
		Password: req.Password,
	})
	if err != nil {
		return err
	}

	c.Response().Header().Set(hxRedirect, "/client/login")
	return c.JSON(http.StatusOK, map[string]interface{}{
		"user_id": userID,
	})
}

func (s *State) LoginUser(c echo.Context) error {
	var req struct {
		Username string `json:"username" validate:"required"`
		Password string `json:"password" validate:"required"`
	}
	if err := c.Bind(&req); err != nil {
		return errBadInput
	}
	if err := validate.Struct(&req); err != nil {
		return err
	}

	userID, err := s.service.LoginUser(c.Request().Context(), req.Username, req.Password)
	if err != nil {
		return err
	}
	if userID == nil {
		return failure.NewError(http.StatusUnauthorized, "invalid username or password")
	}

	token, err := s.auth.GenerateJWT(*userID)
	if err != nil {
		return err
	}

	c.Response().Header().Set(hxRedirect, "/client/timeline")
	c.SetCookie(&http.Cookie{
		Name:     jwtCookieName,
		Value:    token,
		SameSite: http.SameSiteLaxMode,
		Secure:   !s.DevMode(),
		HttpOnly: true,
		MaxAge:   jwtCookieAge,
		Path:     "/",
	})
	return c.JSON(http.StatusOK, map[string]interface{}{
		"user_id": userID,
		"token":   token,
	})
}

func (s *State) LogoutUser(c echo.Context) error {
	type query struct {
		All *bool `query:"all"`
	}

	var q query
	if err := c.Bind(&q); err != nil {
		return errBadInput
	}

	c.SetCookie(&http.Cookie{
		Name:     jwtCookieName,
		SameSite: http.SameSiteLaxMode,
		Secure:   !s.DevMode(),
		HttpOnly: true,
		MaxAge:   -1, // delete cookie
		Path:     "/",
	})

	if q.All != nil && *q.All && isAuthed(c) {
		if err := s.service.LogoutAllUser(c.Request().Context(), c.Get(authCtxName).(*authedUser).UserID); err != nil {
			return err
		}
	}

	c.Response().Header().Set(hxRedirect, "/client/login")
	return c.JSON(http.StatusOK, nil)
}

func (s *State) ClientRegisterUser(c echo.Context) error {
	return c.Render(http.StatusOK, "topRegister.html", nil)
}

func (s *State) ClientLoginUser(c echo.Context) error {
	return c.Render(http.StatusOK, "topLogin.html", nil)
}

type UserOpenGraph struct {
	URL         string
	Title       string
	Description string
	SiteName    string
	Image       string
}

type ClientProfileParams struct {
	Og     UserOpenGraph
	Authed bool
	User   types.DetailedUser
}

func (s *State) ClientProfile(c echo.Context) error {
	userSpecifierStr := c.Param("spec")
	userSpecifier, ok := types.ParseUserSpecifier(userSpecifierStr, s.MyDomain())
	if !ok {
		return errBadInput
	}

	viewerID := getViewerID(c)

	targetUserID, err := s.service.FindUserIDBySpecifierWithRemote(c.Request().Context(), userSpecifier)
	if err != nil {
		return err
	}
	if targetUserID == nil {
		return failure.NewError(http.StatusNotFound, "user not found")
	}

	user, err := s.service.GetUserProfile(c.Request().Context(), viewerID, *targetUserID)
	if err != nil {
		return err
	}

	var avatar string
	if user.Basic.Avatar != nil {
		avatar = user.Basic.Avatar.String()
	}
	params := ClientProfileParams{
		Og: UserOpenGraph{
			URL:         s.BaseURL().JoinPath("client", "user", userSpecifier.String()).String(),
			Title:       user.Basic.Nickname,
			SiteName:    "Lightpub", // TODO
			Description: user.Basic.Bio,
			Image:       avatar,
		},
		Authed: viewerID != nil,
		User:   user,
	}
	return c.Render(http.StatusOK, "topProfile.html", params)
}

func (s *State) ProfileUpdate(c echo.Context) error {
	avatar, err := c.FormFile("avatar")
	if err != nil {
		if errors.Is(err, http.ErrMissingFile) {
			avatar = nil
		} else {
			return err
		}
	}
	avatarRemove := c.FormValue("avatarRemove") == "on"
	nickname := c.FormValue("nickname")
	bio := c.FormValue("bio")
	autoFollowAccept := c.FormValue("autoFollowAccept") == "on"
	hideFollows := c.FormValue("hideFollows") == "on"

	var updatedAvatarUploadID *types.UploadID
	if avatarRemove {
		updatedAvatarUploadID = &types.UploadID{} // zero value means remove
	} else {
		if avatar != nil {
			avatarUploadID, err := s.newUpload(c.Request().Context(), avatar)
			if err != nil {
				return err
			}
			updatedAvatarUploadID = &avatarUploadID // non-nil and non-zero means change
		} else {
			updatedAvatarUploadID = nil // nil means no change
		}
	}

	viewerID := getViewerID(c) // must be non-nil

	if err := s.service.UpdateUserProfile(c.Request().Context(), *viewerID, service.ProfileUpdateParams{
		Nickname:         &nickname,
		Bio:              &bio,
		AvatarUploadID:   updatedAvatarUploadID,
		AutoAcceptFollow: &autoFollowAccept,
		HideFollows:      &hideFollows,
	}); err != nil {
		return err
	}

	c.Response().Header().Set(hxRedirect, "/client/my")
	return c.NoContent(http.StatusOK)
}

type ClientProfileUpdateParams struct {
	User types.DetailedUser
}

func (s *State) ClientProfileUpdatePage(c echo.Context) error {
	viewerID := getViewerID(c) // must be non-nil

	user, err := s.service.GetUserProfile(c.Request().Context(), viewerID, *viewerID)
	if err != nil {
		return err
	}

	params := ClientProfileUpdateParams{
		User: user,
	}
	return c.Render(http.StatusOK, "topProfileUpdate.html", params)
}

func (s *State) ClientMy(c echo.Context) error {
	viewerID := getViewerID(c) // must be non-nil
	return c.Redirect(http.StatusTemporaryRedirect, s.BaseURL().JoinPath("client", "user", viewerID.String()).String())
}

func (s *State) GetUserAvatar(c echo.Context) error {
	userIDStr := c.Param("id")
	userID, err := types.ParseUserID(userIDStr)
	if err != nil {
		return err
	}

	user, err := s.service.FindUserByID(c.Request().Context(), userID)
	if err != nil {
		return err
	}
	if user == nil {
		return failure.NewError(http.StatusNotFound, "user not found")
	}

	avatar, err := s.service.GetUserAvatarFromUser(*user)
	if err != nil {
		return err
	}

	if avatar.HasUpload {
		return c.Redirect(http.StatusTemporaryRedirect, s.BaseURL().JoinPath("upload", avatar.UploadID.String()).String())
	} else {
		return c.Blob(http.StatusOK, "image/jpeg", avatar.Ideticon)
	}
}

type UserListParams struct {
	Data    []UserListEntry
	NextURL string
}

type UserListEntry struct {
	User      types.SimpleUser
	CreatedAt *time.Time
}

func makeUserListParamsFromSlice(users []types.SimpleUser, limit int, nextURL string) UserListParams {
	if len(users) > limit {
		users = users[:limit]
	}

	params := make([]UserListEntry, len(users))
	for i, user := range users {
		params[i].User = user
	}
	return UserListParams{
		Data:    params,
		NextURL: nextURL,
	}
}

func (s *State) GetUserFollowings(c echo.Context) error {
	id := c.Param("id")
	userID, err := types.ParseUserID(id)
	if err != nil {
		return errBadInput
	}
	var query struct {
		Page int `query:"page"`
	}

	followings, err := s.service.GetUserFollowingList(c.Request().Context(), userID, paginationSizeP1, query.Page)
	if err != nil {
		return err
	}

	var nextURL string
	if len(followings) == paginationSizeP1 {
		nextURL = buildURLWithParams(c.Request().URL, map[string]string{
			"page": strconv.Itoa(query.Page + 1),
		})
	}

	params := makeUserListParamsFromSlice(followings, paginationSize, nextURL)
	return c.Render(http.StatusOK, "userList.html", params)
}

func (s *State) GetUserFollowers(c echo.Context) error {
	id := c.Param("id")
	userID, err := types.ParseUserID(id)
	if err != nil {
		return errBadInput
	}
	var query struct {
		Page int `query:"page"`
	}

	followers, err := s.service.GetUserFollowersList(c.Request().Context(), userID, paginationSizeP1, query.Page)
	if err != nil {
		return err
	}

	var nextURL string
	if len(followers) == paginationSizeP1 {
		nextURL = buildURLWithParams(c.Request().URL, map[string]string{
			"page": strconv.Itoa(query.Page + 1),
		})
	}

	params := makeUserListParamsFromSlice(followers, paginationSize, nextURL)
	return c.Render(http.StatusOK, "userList.html", params)
}

type ClientUserListParams struct {
	Title string
	URL   string
}

func (s *State) ClientUserFollowings(c echo.Context) error {
	userSpecStr := c.Param("spec")
	userSpec, ok := types.ParseUserSpecifier(userSpecStr, s.MyDomain())
	if !ok {
		return errBadInput
	}

	userID, err := s.service.FindLocalUserIDBySpecifier(c.Request().Context(), userSpec)
	if err != nil {
		return errBadInput
	}
	if userID == nil {
		return failure.NewError(http.StatusNotFound, "user not found")
	}

	return c.Render(http.StatusOK, "topUserList.html", ClientUserListParams{
		Title: "フォロー一覧",
		URL:   "/user/" + userID.String() + "/following",
	})
}

func (s *State) ClientUserFollowers(c echo.Context) error {
	userSpecStr := c.Param("spec")
	userSpec, ok := types.ParseUserSpecifier(userSpecStr, s.MyDomain())
	if !ok {
		return errBadInput
	}

	userID, err := s.service.FindLocalUserIDBySpecifier(c.Request().Context(), userSpec)
	if err != nil {
		return errBadInput
	}
	if userID == nil {
		return failure.NewError(http.StatusNotFound, "user not found")
	}

	return c.Render(http.StatusOK, "topUserList.html", ClientUserListParams{
		Title: "フォロワ―一覧",
		URL:   "/user/" + userID.String() + "/followers",
	})
}

type UserInteractionType string

const (
	userInteractionFollow       = "follow"
	userInteractionUnfollow     = "unfollow"
	userInteractionBlock        = "block"
	userInteractionUnblock      = "unblock"
	userInteractionAcceptFollow = "acceptFollow"
	userInteractionRejectFollow = "rejectFollow"
)

func (s *State) UserInteraction(c echo.Context) error {
	viewerID := getViewerID(c) // must be non-nil

	var req struct {
		Type UserInteractionType `json:"type" validate:"required,oneof=follow unfollow block unblock acceptFollow rejectFollow"`
	}

	if err := c.Bind(&req); err != nil {
		return errBadInput
	}
	if err := validate.Struct(&req); err != nil {
		return errBadInput
	}

	targetUserIDStr := c.Param("id")
	targetUserID, err := types.ParseUserID(targetUserIDStr)
	if err != nil {
		return err
	}

	switch req.Type {
	case userInteractionFollow:
		err = s.service.FollowUser(c.Request().Context(), *viewerID, targetUserID)
	case userInteractionUnfollow:
		err = s.service.UnfollowUser(c.Request().Context(), *viewerID, targetUserID)
	case userInteractionBlock:
		err = s.service.BlockUser(c.Request().Context(), *viewerID, targetUserID)
	case userInteractionUnblock:
		err = s.service.UnblockUser(c.Request().Context(), *viewerID, targetUserID)
	case userInteractionAcceptFollow:
		err = s.service.AcceptFollow(c.Request().Context(), *viewerID, targetUserID)
	case userInteractionRejectFollow:
		err = s.service.RejectFollowUser(c.Request().Context(), *viewerID, targetUserID)
	default:
		return failure.NewError(http.StatusBadRequest, "invalid interaction type")
	}

	if err != nil {
		return err
	}

	c.Response().Header().Set(hxRefresh, trueHeaderValue)
	return c.NoContent(http.StatusOK)
}
