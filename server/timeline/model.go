package timeline

import "time"

type FetchedPost struct {
	ID             string    `db:"id" json:"i"`
	PosterID       string    `db:"poster_id" json:"pi"`
	PosterUsername string    `db:"poster_username" json:"pu"`
	Content        string    `db:"content" json:"c"`
	CreatedAt      time.Time `db:"created_at" json:"t"`
	Privacy        string    `db:"privacy" json:"pv"`
}
