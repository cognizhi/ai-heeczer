FROM node:22-alpine AS sdk
WORKDIR /repo/bindings/heeczer-js
COPY bindings/heeczer-js/package.json bindings/heeczer-js/package-lock.json* ./
RUN npm install
COPY bindings/heeczer-js/tsconfig*.json ./
COPY bindings/heeczer-js/scripts ./scripts
COPY bindings/heeczer-js/src ./src
COPY bindings/heeczer-js/README.md ./README.md
RUN npm run build

FROM node:22-alpine AS app
WORKDIR /repo/testing/compose/js/chatbot
COPY testing/compose/js/chatbot/package.json testing/compose/js/chatbot/tsconfig.json ./
COPY --from=sdk /repo/bindings/heeczer-js /repo/bindings/heeczer-js
RUN npm install
COPY testing/compose/js/chatbot/src ./src
COPY testing/tests/fixtures /fixtures
RUN npm run build
EXPOSE 8000
CMD ["node", "dist/server.js"]
