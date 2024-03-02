package pub

import (
	"errors"
	"net/url"

	"github.com/go-fed/activity/streams"
	"github.com/go-fed/activity/streams/vocab"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/posts"
)

type PubPostService struct {
	getter IDGetterService
}

func ProvidePubPostService(getter IDGetterService) *PubPostService {
	return &PubPostService{getter: getter}
}

func (s *PubPostService) CreatePostObject(post *db.Post) (vocab.ActivityStreamsNote, error) {
	postURI, err := s.getter.GetPostID(post)
	if err != nil {
		return nil, err
	}
	posterURI, err := s.getter.GetUserID(&post.Poster, "")
	if err != nil {
		return nil, err
	}

	// Create the post object
	obj := streams.NewActivityStreamsNote()

	noteID := streams.NewJSONLDIdProperty()
	noteID.Set(postURI)
	obj.SetJSONLDId(noteID)

	attributedTo := streams.NewActivityStreamsAttributedToProperty()
	attributedTo.AppendIRI(posterURI)
	obj.SetActivityStreamsAttributedTo(attributedTo)

	toAndCc, err := s.calculateToAndCc(post)
	if err != nil {
		return nil, err
	}
	to := streams.NewActivityStreamsToProperty()

	return obj, nil
}

type toAndCc struct {
	To []vocab.ActivityStreamsObject
	Cc []vocab.ActivityStreamsObject
}

func (s *PubPostService) calculateToAndCc(post *db.Post) (toAndCc, error) {
	to := make([]vocab.ActivityStreamsObject, 0)
	cc := make([]vocab.ActivityStreamsObject, 0)

	if post.Privacy == "" {
		return toAndCc{}, errors.New("post privacy is empty")
	}
	if post.Poster.ID == (db.UUID{}) {
		return toAndCc{}, errors.New("post poster id is empty")
	}

	posterFollowers, err := s.getter.GetUserID(&post.Poster, "followers")
	if err != nil {
		return toAndCc{}, err
	}

	publicURI := streams.NewActivityStreamsObject()
	publicID := streams.NewJSONLDIdProperty()
	publicID.SetIRI(urlMustParse("https://www.w3.org/ns/activitystreams#Public"))
	publicURI.SetJSONLDId(publicID)

	followersURI := streams.NewActivityStreamsObject()
	followersID := streams.NewJSONLDIdProperty()
	followersID.SetIRI(posterFollowers)
	followersURI.SetJSONLDId(followersID)

	privacy := posts.PrivacyType(post.Privacy)
	switch privacy {
	case posts.PrivacyPublic:
		to = append(to, publicURI)
		cc = append(cc, followersURI)
	case posts.PrivacyUnlisted:
		to = append(to, followersURI)
		cc = append(cc, publicURI)
	case posts.PrivacyFollower:
		to = append(to, followersURI)
	case posts.PrivacyPrivate:
		// TODO: mentioned users
	}

	return toAndCc{To: to, Cc: cc}, nil
}

func urlMustParse(s string) *url.URL {
	u, err := url.Parse(s)
	if err != nil {
		panic(err)
	}
	return u
}
