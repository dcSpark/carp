FROM node:16.15.0

WORKDIR /app
COPY ./ /app
WORKDIR /app/server
RUN yarn install
RUN yarn build

ENTRYPOINT ["yarn", "prod:start"]
