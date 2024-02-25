# Lightpub

## Server 建て方
1. `cd server`
2. `pipenv install`
3. `docker run -d --name lightpub-db -e MYSQL_USER=lightpub -e MYSQL_PASSWORD=lightpub -e MYSQL_ROOT_PASSWORD=lightpub -e MYSQL_DATABASE=lightpub -p 3306:3306 mariadb:latest`
4. `./manage.py migrate`
5. `./manage.py runserver`

## Frontend 建て方
1. `cd frontend`
2. yarn install
3. yarn run dev
