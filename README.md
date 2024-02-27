# Lightpub

## Server 建て方
1. `cd server`
2. `pipenv install`
3. `docker run -d --name lightpub-db -e MYSQL_USER=lightpub -e MYSQL_PASSWORD=lightpub -e MYSQL_ROOT_PASSWORD=lightpub -e MYSQL_DATABASE=lightpub -p 3306:3306 mariadb:latest`
4. `./manage.py migrate`
5. `./manage.py runserver`

完成を急ぐため Python で書いていますが、一通り実装し終わった後、性能次第では Rust などのより"軽量"な言語に移行します。

## Frontend 建て方
1. `cd frontend`
2. `yarn install`
3. `yarn run dev`
