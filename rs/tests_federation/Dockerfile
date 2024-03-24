FROM node:18

# RUN apt-get update -qq && apt-get install -y -qq mariadb-client

WORKDIR /app
COPY package.json package-lock.json ./
RUN npm install

COPY . .

CMD ["npm", "run", "test"]
