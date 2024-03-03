package pub

import (
	"errors"
	"log"
	"net/url"

	"github.com/go-fed/activity/streams"
	"github.com/go-fed/activity/streams/vocab"
	"github.com/lightpub-dev/lightpub/db"
	"github.com/lightpub-dev/lightpub/posts"
	"github.com/lightpub-dev/lightpub/users"
)

type PubPostService struct {
	getter     IDGetterService
	userFinder users.UserFinderService
	userFollow users.UserFollowService
}

func ProvidePubPostService(getter IDGetterService, userFinder users.UserFinderService, userFollow users.UserFollowService) *PubPostService {
	return &PubPostService{getter: getter, userFinder: userFinder, userFollow: userFollow}
}

type PubNoteInfo struct {
	Post          *db.Post
	Note          vocab.ActivityStreamsNote
	TargetInboxes []*url.URL
}

type PubCreateInfo struct {
	Post          *db.Post
	Create        vocab.ActivityStreamsCreate
	TargetInboxes []*url.URL
}

type PubAnnounceInfo struct {
	Post          *db.Post
	Announce      vocab.ActivityStreamsAnnounce
	TargetInboxes []*url.URL
}

func (s *PubPostService) CreateNoteCreate(note PubNoteInfo) (PubCreateInfo, error) {
	obj := streams.NewActivityStreamsCreate()

	createID := streams.NewJSONLDIdProperty()
	createPostID, err := s.getter.GetPostID(note.Post, "activity")
	if err != nil {
		return PubCreateInfo{}, err
	}
	createID.Set(createPostID)
	obj.SetJSONLDId(createID)

	copyAddressing(obj, note.Note)

	publishedAt := streams.NewActivityStreamsPublishedProperty()
	publishedAt.Set(note.Post.CreatedAt)
	obj.SetActivityStreamsPublished(publishedAt)

	copyActor(obj, note.Note)

	noteObj := streams.NewActivityStreamsObjectProperty()
	noteObj.AppendActivityStreamsNote(note.Note)
	obj.SetActivityStreamsObject(noteObj)

	return PubCreateInfo{
		Post:          note.Post,
		Create:        obj,
		TargetInboxes: note.TargetInboxes,
	}, nil
}

func copyActor(dst vocab.ActivityStreamsCreate, src vocab.ActivityStreamsNote) {
	actorIter := src.GetActivityStreamsAttributedTo().Begin()
	for actorIter != nil {
		dst.GetActivityStreamsActor().AppendActivityStreamsObject(actorIter.GetActivityStreamsObject())
		actorIter = actorIter.Next()
	}
}

func copyAddressing(dst vocab.ActivityStreamsCreate, src vocab.ActivityStreamsNote) {
	to := src.GetActivityStreamsTo()
	toIter := to.Begin()
	for toIter != nil {
		dst.GetActivityStreamsTo().AppendActivityStreamsObject(toIter.GetActivityStreamsObject())
		toIter = toIter.Next()
	}

	cc := src.GetActivityStreamsCc()
	ccIter := cc.Begin()
	for ccIter != nil {
		dst.GetActivityStreamsCc().AppendActivityStreamsObject(ccIter.GetActivityStreamsObject())
		ccIter = ccIter.Next()
	}

	bto := src.GetActivityStreamsBto()
	btoIter := bto.Begin()
	for btoIter != nil {
		dst.GetActivityStreamsBto().AppendActivityStreamsObject(btoIter.GetActivityStreamsObject())
		btoIter = btoIter.Next()
	}

	bcc := src.GetActivityStreamsBcc()
	bccIter := bcc.Begin()
	for bccIter != nil {
		dst.GetActivityStreamsBcc().AppendActivityStreamsObject(bccIter.GetActivityStreamsObject())
		bccIter = bccIter.Next()
	}
}

func (s *PubPostService) CreateAnnounce(post *db.Post) (PubAnnounceInfo, error) {
	// check if this is a repost
	if !post.RepostOfID.Valid {
		return PubAnnounceInfo{}, errors.New("cannot create an Announce out from a non-repost")
	}
	if post.Content.Valid {
		return PubAnnounceInfo{}, errors.New("cannot create an Announce out from a quote")
	}

	announce := streams.NewActivityStreamsAnnounce()

	announceID, err := s.getter.GetPostID(post, "activity")
	if err != nil {
		return PubAnnounceInfo{}, err
	}
	announceIDProp := streams.NewJSONLDIdProperty()
	announceIDProp.Set(announceID)
	announce.SetJSONLDId(announceIDProp)

	toAndCc, err := s.calculateToAndCc(post)
	if err != nil {
		return PubAnnounceInfo{}, err
	}
	toProp := streams.NewActivityStreamsToProperty()
	for _, to := range toAndCc.To {
		toProp.AppendActivityStreamsObject(to)
	}
	announce.SetActivityStreamsTo(toProp)
	ccProp := streams.NewActivityStreamsCcProperty()
	for _, cc := range toAndCc.Cc {
		ccProp.AppendActivityStreamsObject(cc)
	}
	announce.SetActivityStreamsCc(ccProp)

	publishedProp := streams.NewActivityStreamsPublishedProperty()
	publishedProp.Set(post.CreatedAt)
	announce.SetActivityStreamsPublished(publishedProp)

	repostedObjectID, err := s.getter.GetPostID(post.RepostOf, "")
	if err != nil {
		return PubAnnounceInfo{}, err
	}
	repostedObjectProp := streams.NewActivityStreamsObjectProperty()
	repostedObjectProp.AppendIRI(repostedObjectID)
	announce.SetActivityStreamsObject(repostedObjectProp)

	return PubAnnounceInfo{
		Post:          post,
		Announce:      announce,
		TargetInboxes: toAndCc.TargetInboxes,
	}, nil
}

func (s *PubPostService) CreatePostObject(post *db.Post) (PubNoteInfo, error) {
	// check that this is not a repost (repostId != null and content == null)
	if post.RepostOfID.Valid && !post.Content.Valid {
		return PubNoteInfo{}, errors.New("cannot create a Note out from a repost")
	}

	postURI, err := s.getter.GetPostID(post, "")
	if err != nil {
		return PubNoteInfo{}, err
	}
	posterURI, err := s.getter.GetUserID(&post.Poster, "")
	if err != nil {
		return PubNoteInfo{}, err
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
		return PubNoteInfo{}, err
	}
	to := streams.NewActivityStreamsToProperty()
	for _, t := range toAndCc.To {
		to.AppendActivityStreamsObject(t)
	}
	cc := streams.NewActivityStreamsCcProperty()
	for _, c := range toAndCc.Cc {
		cc.AppendActivityStreamsObject(c)
	}
	obj.SetActivityStreamsTo(to)
	obj.SetActivityStreamsCc(cc)

	content := streams.NewActivityStreamsContentProperty()
	content.AppendXMLSchemaString(post.Content.String)
	obj.SetActivityStreamsContent(content)

	published := streams.NewActivityStreamsPublishedProperty()
	published.Set(post.CreatedAt)
	obj.SetActivityStreamsPublished(published)

	if post.ReplyToID.Valid {
		replyToURI, err := s.getter.GetPostID(post.ReplyTo, "")
		if err != nil {
			return PubNoteInfo{}, err
		}
		replyTo := streams.NewActivityStreamsInReplyToProperty()
		replyTo.AppendIRI(replyToURI)
		obj.SetActivityStreamsInReplyTo(replyTo)
	}

	sense := streams.NewActivityStreamsSensitiveProperty()
	sense.AppendXMLSchemaBoolean(false)
	obj.SetActivityStreamsSensitive(sense)

	return PubNoteInfo{
		Note:          obj,
		TargetInboxes: toAndCc.TargetInboxes,
	}, nil
}

type toAndCc struct {
	To            []vocab.ActivityStreamsObject
	Cc            []vocab.ActivityStreamsObject
	TargetInboxes []*url.URL
}

func (s *PubPostService) findBestInbox(user users.FollowerInbox) (*url.URL, error) {
	if user.SharedInbox.Valid {
		url, err := url.Parse(user.SharedInbox.String)
		if err != nil {
			log.Printf("invalid sharedInbox: %s", user.SharedInbox.String)
			return nil, nil
		}
		return url, nil
	}
	if user.Inbox.Valid {
		url, err := url.Parse(user.Inbox.String)
		if err != nil {
			log.Printf("invalid inbox: %s", user.Inbox.String)
			return nil, nil
		}
		return url, nil
	}
	return nil, nil
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
	targetFollowers := false
	switch privacy {
	case posts.PrivacyPublic:
		to = append(to, publicURI)
		cc = append(cc, followersURI)
		targetFollowers = true
	case posts.PrivacyUnlisted:
		to = append(to, followersURI)
		cc = append(cc, publicURI)
		targetFollowers = true
	case posts.PrivacyFollower:
		to = append(to, followersURI)
		targetFollowers = true
	case posts.PrivacyPrivate:
		// TODO: mentioned users
	}

	targetInboxes := make([]*url.URL, 0)
	if targetFollowers {
		followers, err := s.userFollow.FindFollowersInboxes(post.Poster.ID)
		if err != nil {
			return toAndCc{}, err
		}
		for _, f := range followers {
			inbox, err := s.findBestInbox(f)
			if err != nil {
				return toAndCc{}, err
			}
			if inbox != nil {
				targetInboxes = append(targetInboxes, inbox)
			}
		}
	}

	return toAndCc{To: to, Cc: cc, TargetInboxes: targetInboxes}, nil
}

func urlMustParse(s string) *url.URL {
	u, err := url.Parse(s)
	if err != nil {
		panic(err)
	}
	return u
}
