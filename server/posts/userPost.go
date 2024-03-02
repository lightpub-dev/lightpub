package posts

import (
	"sort"
	"time"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/users"
)

type UserPostFetchOptions struct {
	Limit         int
	CreatedBefore *time.Time
}

type UserPost struct {
	models.UserPostEntry
	ReplyToID  db.NullUUID
	RepostOfID db.NullUUID
}

func (p *UserPost) FillInteractions(repostedByMe, favoritedByMe, bookmarkedByMe bool) {
	p.RepostedByMe = &repostedByMe
	p.FavoritedByMe = &favoritedByMe
	p.BookmarkedByMe = &bookmarkedByMe
}

type UserPostService interface {
	GetUserPosts(userID db.UUID, viewerID db.UUID, options UserPostFetchOptions) ([]UserPost, error)
}

type DBUserPostService struct {
	conn        db.DBConn
	interaction PostInteractionService
	follow      users.UserFollowService
}

func ProvideDBUserPostService(conn db.DBConn, interaction PostInteractionService, follow users.UserFollowService) *DBUserPostService {
	return &DBUserPostService{conn, interaction, follow}
}

func (s *DBUserPostService) GetUserPosts(targetUserID db.UUID, viewerUserID db.UUID, options UserPostFetchOptions) ([]UserPost, error) {
	conn := s.conn.DB
	limit := options.Limit

	// first, get all "public" and "unlisted" posts
	var publicPosts []UserPost
	publicPostsQuery := conn.Table("posts p").Where("p.poster_id = ? AND p.privacy IN ('public','unlisted')", targetUserID).Order("p.created_at DESC").Limit(limit).Select("p.id, p.content, p.created_at, p.privacy, p.reply_to_id, p.repost_of_id, NULL AS reposted_by_me, NULL AS favorited_by_me, NULL AS bookmarked_by_me")
	err := publicPostsQuery.Scan(&publicPosts).Error
	if err != nil {
		return nil, err
	}
	if viewerUserID != (db.UUID{}) {
		// if viewer is logged in, fetch repostedByMe, favoritedByMe, etc...
		for i := range publicPosts {
			err := s.interaction.FillInteraction(viewerUserID, &publicPosts[i])
			if err != nil {
				return nil, err
			}
		}
	}

	// "follower" posts...
	var followerPosts []UserPost
	if viewerUserID != (db.UUID{}) {
		isFollowed := false
		if viewerUserID == targetUserID {
			// when viewer is target itself...
			isFollowed = true
		}
		if !isFollowed {
			// check if user is followed by target
			isFollowed, err = s.follow.IsFollowedBy(viewerUserID, targetUserID)
			if err != nil {
				return nil, err
			}
		}
		if isFollowed {
			// fetch "follower" posts
			// TODO
			// 	err = h.DB.Select(&followerPosts, `
			// SELECT BIN_TO_UUID(p.id) AS id,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id,
			// IF(?='', NULL, (SELECT COUNT(*) > 0 FROM Post p2 WHERE p2.repost_of=p.id AND p2.poster_id=UUID_TO_BIN(IF(?='',NULL,?)) AND p2.content IS NULL)) AS reposted_by_me,
			// IF(?='', NULL, (SELECT COUNT(*) > 0 FROM PostFavorite pf WHERE pf.post_id=p.id AND pf.user_id=UUID_TO_BIN(IF(?='',NULL,?)) AND pf.is_bookmark=0)) AS favorited_by_me,
			// IF(?='', NULL, (SELECT COUNT(*) > 0 FROM PostFavorite pf WHERE pf.post_id=p.id AND pf.user_id=UUID_TO_BIN(IF(?='',NULL,?)) AND pf.is_bookmark=1)) AS bookmarked_by_me
			// FROM Post p
			// WHERE
			// 	p.poster_id=UUID_TO_BIN(?)
			// 	AND p.privacy = 'follower'
			// 	AND p.scheduled_at IS NULL
			// ORDER BY p.created_at DESC
			// LIMIT ?
			// `, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, targetUser.ID, limit)
			// 	if err != nil {
			// 		c.Logger().Error(err)
			// 		return c.String(500, "internal server error")
			// 	}
		}
	}

	// fetch "private" posts
	var privatePosts []UserPost
	if viewerUserID != (db.UUID{}) {
		if targetUserID == viewerUserID {
			// when viewer is target itself, fetch all private posts
			// TODO
			// 	err = h.DB.Select(&privatePosts, `
			// 	SELECT BIN_TO_UUID(p.id) AS id,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id,
			// 	IF(?='', NULL, (SELECT COUNT(*) > 0 FROM Post p2 WHERE p2.repost_of=p.id AND p2.poster_id=UUID_TO_BIN(IF(?='',NULL,?)) AND p2.content IS NULL)) AS reposted_by_me,
			// 	IF(?='', NULL, (SELECT COUNT(*) > 0 FROM PostFavorite pf WHERE pf.post_id=p.id AND pf.user_id=UUID_TO_BIN(IF(?='',NULL,?)) AND pf.is_bookmark=0)) AS favorited_by_me,
			// IF(?='', NULL, (SELECT COUNT(*) > 0 FROM PostFavorite pf WHERE pf.post_id=p.id AND pf.user_id=UUID_TO_BIN(IF(?='',NULL,?)) AND pf.is_bookmark=1)) AS bookmarked_by_me
			// FROM Post p
			// WHERE
			// 	p.poster_id=UUID_TO_BIN(?)
			// 	AND p.privacy = 'private'
			// 	AND p.scheduled_at IS NULL
			// ORDER BY p.created_at DESC
			// LIMIT ?
			// `, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, targetUser.ID, limit)
		} else {
			// TODO
			// 		err = h.DB.Select(&privatePosts, `
			// SELECT BIN_TO_UUID(p.id) AS id,p.content,p.created_at,p.privacy,BIN_TO_UUID(p.reply_to) AS reply_to,BIN_TO_UUID(p.repost_of) AS repost_of,BIN_TO_UUID(p.poll_id) AS poll_id,
			// IF(?='', NULL, (SELECT COUNT(*) > 0 FROM Post p2 WHERE p2.repost_of=p.id AND p2.poster_id=UUID_TO_BIN(IF(?='',NULL,?)) AND p2.content IS NULL)) AS reposted_by_me,
			// IF(?='', NULL, (SELECT COUNT(*) > 0 FROM PostFavorite pf WHERE pf.post_id=p.id AND pf.user_id=UUID_TO_BIN(IF(?='',NULL,?)) AND pf.is_bookmark=0)) AS favorited_by_me,
			// 	IF(?='', NULL, (SELECT COUNT(*) > 0 FROM PostFavorite pf WHERE pf.post_id=p.id AND pf.user_id=UUID_TO_BIN(IF(?='',NULL,?)) AND pf.is_bookmark=1)) AS bookmarked_by_me
			// FROM Post p
			// INNER JOIN PostMention pm ON p.id=pm.post_id
			// WHERE
			// 	p.poster_id=UUID_TO_BIN(?)
			// 	AND p.privacy = 'private'
			// 	AND p.scheduled_at IS NULL
			// 	AND pm.target_user_id=UUID_TO_BIN(?)
			// ORDER BY p.created_at DESC
			// LIMIT ?
			// `, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, viewerUserID, targetUser.ID, viewerUserID, limit)
		}
		err = nil
		if err != nil {
			return nil, err
		}
	}

	// merge these allPosts
	allPosts := append(publicPosts, followerPosts...)
	allPosts = append(allPosts, privatePosts...)
	// sort by created_at desc
	sort.Slice(allPosts, func(i int, j int) bool {
		createdAtI := allPosts[i].CreatedAt
		createdAtJ := allPosts[j].CreatedAt
		return createdAtI.After(createdAtJ) // DESC
	})
	// limit to limit
	if len(allPosts) > limit {
		allPosts = allPosts[:limit]
	}

	return allPosts, nil
}
