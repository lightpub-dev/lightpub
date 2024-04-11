FROM node:lts as builder

WORKDIR /usr/src/app

# Install dependencies
COPY package.json yarn.lock ./
RUN yarn install

# Build bundle
COPY . .
ARG API_URL

RUN rm -rf dist && env API_URL=$API_URL yarn run build

FROM httpd:alpine AS server
COPY ./docker/httpd.conf /usr/local/apache2/conf/httpd.conf
RUN mkdir /usr/local/apache2/htdocs/web
COPY --from=builder /usr/src/app/dist /usr/local/apache2/htdocs/web

EXPOSE 80
