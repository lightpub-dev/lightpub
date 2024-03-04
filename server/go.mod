module github.com/lightpub-dev/lightpub

go 1.20

require (
	github.com/go-fed/activity v1.0.0
	github.com/go-fed/httpsig v0.1.1-0.20190914113940-c2de3672e5b5
	github.com/go-playground/validator/v10 v10.16.0
	github.com/go-sql-driver/mysql v1.7.1
	github.com/golang-migrate/migrate/v4 v4.17.0
	github.com/google/uuid v1.5.0
	github.com/google/wire v0.6.0
	github.com/jmoiron/sqlx v1.3.5
	github.com/k0kubun/pp/v3 v3.2.0
	github.com/labstack/echo/v4 v4.11.4
	github.com/redis/go-redis/v9 v9.3.1
	github.com/stretchr/testify v1.8.4
	golang.org/x/crypto v0.18.0
	gorm.io/driver/mysql v1.5.2
	gorm.io/gorm v1.25.5
)

require (
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/hashicorp/errwrap v1.1.0 // indirect
	github.com/hashicorp/go-multierror v1.1.1 // indirect
	github.com/jinzhu/inflection v1.0.0 // indirect
	github.com/jinzhu/now v1.1.5 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/swaggo/files/v2 v2.0.0 // indirect
	go.uber.org/atomic v1.7.0 // indirect
)

require (
	github.com/KyleBanks/depth v1.2.1 // indirect
	github.com/cespare/xxhash/v2 v2.2.0 // indirect
	github.com/dgryski/go-rendezvous v0.0.0-20200823014737-9f7001d12a5f // indirect
	github.com/gabriel-vasile/mimetype v1.4.2 // indirect
	github.com/go-openapi/jsonpointer v0.20.2 // indirect
	github.com/go-openapi/jsonreference v0.20.4 // indirect
	github.com/go-openapi/spec v0.20.13 // indirect
	github.com/go-openapi/swag v0.22.7 // indirect
	github.com/go-playground/locales v0.14.1 // indirect
	github.com/go-playground/universal-translator v0.18.1 // indirect
	github.com/golang-jwt/jwt v3.2.2+incompatible // indirect
	github.com/josharian/intern v1.0.0 // indirect
	github.com/labstack/gommon v0.4.2
	github.com/leodido/go-urn v1.2.4 // indirect
	github.com/mailru/easyjson v0.7.7 // indirect
	github.com/mattn/go-colorable v0.1.13 // indirect
	github.com/mattn/go-isatty v0.0.20 // indirect
	github.com/swaggo/http-swagger/v2 v2.0.2
	github.com/swaggo/swag v1.16.2 // indirect
	github.com/valyala/bytebufferpool v1.0.0 // indirect
	github.com/valyala/fasttemplate v1.2.2 // indirect
	golang.org/x/net v0.20.0 // indirect
	golang.org/x/sys v0.16.0 // indirect
	golang.org/x/text v0.14.0 // indirect
	golang.org/x/time v0.5.0 // indirect
	golang.org/x/tools v0.17.0 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

replace github.com/go-fed/activity v1.0.0 => ./activity

replace github.com/go-fed/httpsig => ./httpsig
