package posts

import (
	"context"

	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/models"
	"github.com/lightpub-dev/lightpub/timeline"
	"github.com/lightpub-dev/lightpub/users"
	"github.com/redis/go-redis/v9"
)

func RegisterToTimeline(ctx context.Context, tx db.DBOrTx, rdb *redis.Client, post models.Post, posterUsername string, hashtags []string, mentions []string) error {
	loaclReceiverIDs := []string{}
	remoteReceiverIDs := []string{}

	// TODO: add mentioned users here

	// poster is always a receiver
	loaclReceiverIDs = append(loaclReceiverIDs, post.PosterID)

	switch post.Privacy {
	case PrivacyUnlisted:
		// receiver is poster only
		break
	case PrivacyPublic:
		fallthrough
	case PrivacyFollower:
		// add followers
		followers, err := users.FindFollowers(ctx, tx, post.PosterID)
		if err != nil {
			return err
		}
		for _, follower := range followers {
			if follower.Host == "" {
				// local
				loaclReceiverIDs = append(loaclReceiverIDs, follower.ID)
			} else {
				// remote
				remoteReceiverIDs = append(remoteReceiverIDs, follower.ID)
			}
		}
		break
	case PrivacyPrivate:
		// receiver is mentioned users only
		// mentioned users are already registered above.
		break
	}

	targetPost := timeline.FetchedPost{
		ID:             post.ID,
		PosterID:       post.PosterID,
		PosterUsername: posterUsername,
		Content:        post.Content,
		CreatedAt:      post.CreatedAt,
		Privacy:        post.Privacy,
	}

	// process local receivers
	for _, receiverID := range loaclReceiverIDs {
		if err := timeline.AddToTimeline(ctx, tx, rdb, receiverID, targetPost); err != nil {
			return err
		}
	}

	// TODO: process remote receivers

	return nil
}
