FROM node:8.7-alpine

COPY package.json package.json
RUN npm install

COPY lock.js lock.js
COPY elector.js elector.js

CMD node lock.js
