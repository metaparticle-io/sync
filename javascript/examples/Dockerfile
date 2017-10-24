FROM node:8.7-alpine

COPY lock.js lock.js
COPY elector.js elector.js
COPY package.json package.json

RUN npm install

CMD node lock.js
