FROM node:22-alpine AS build

RUN corepack enable && corepack prepare pnpm@latest --activate
RUN apk add --no-cache protobuf protobuf-dev

WORKDIR /app

ENV PATH="/app/node_modules/.bin:$PATH"

COPY apps/frontend/package.json apps/frontend/pnpm-lock.yaml ./
RUN echo "node-linker=hoisted" > .npmrc && pnpm install --no-frozen-lockfile

COPY libs/protocol/proto/ /proto/
COPY apps/frontend/ .

RUN rm -rf src/generated && mkdir -p src/generated && \
    protoc -I=/proto --ts_out=src/generated /proto/*.proto

ARG VITE_API_URL=""
ENV VITE_API_URL=$VITE_API_URL

RUN pnpm run extract && pnpm run compile && npx vite build

FROM nginx:alpine

COPY --from=build /app/dist /usr/share/nginx/html
COPY apps/frontend/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
