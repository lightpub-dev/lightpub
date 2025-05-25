# Lightpub

Lightpub is a lightweight ActivityPub-compliant SNS server written in Rust.

## Goals

- Lightweight server
- Lightweight and simple frontend
- Easy to deploy
- Emoji reactions (compatible with Misskey)
- Rich text editing (markdown, latex, etc)
- Good frontend accessibility

## Technologies

Database: MariaDB, Redis
Web server: Rust, actix-web
Web frontend: Bootstrap, htmx, alpine.js, font-awesome
Deploy: Docker, GNU Make


## Conventions
In frontend development, HTML, css, and css are saved as separated files (except those embedded within html with htmx or alpine.js)
