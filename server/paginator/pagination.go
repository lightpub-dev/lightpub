package paginator

type PaginatedResponse[T any] struct {
	Next     string `json:"next"`
	Previous string `json:"previous"`
	Results  T      `json:"results"`
}

// func Paginate[T any](results []T)
