package apub

const (
	PublicURL = "https://www.w3.org/ns/activitystreams#Public"
)

var (
	publicURLs = []string{
		PublicURL,
		"Public",
		"as:Public",
	}
	followersSuffix = "/followers"
)

type URI = string

func containsPublicURL(urls []string) bool {
	for _, url := range urls {
		for _, publicURL := range publicURLs {
			if url == publicURL {
				return true
			}
		}
	}
	return false
}

type Object interface {
	// ID returns the ID of the object.
	ID() string
}

type Actor interface {
	// PublicKey returns the public key of the actor.
	PublicKey() string
}

type Identifiable[T Object] interface {
}
