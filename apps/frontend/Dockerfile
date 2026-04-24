FROM node:22-alpine AS build

RUN corepack enable && corepack prepare pnpm@latest --activate

WORKDIR /app

ENV PATH="/app/node_modules/.bin:$PATH"

COPY apps/frontend/package.json apps/frontend/pnpm-lock.yaml ./
RUN pnpm install --no-frozen-lockfile

COPY crates/scylla-protocol/proto/ ../../crates/scylla-protocol/proto/
COPY apps/frontend/ .

ARG VITE_API_URL=""
ENV VITE_API_URL=$VITE_API_URL

RUN pnpm run build

FROM nginx:alpine

COPY --from=build /app/dist /usr/share/nginx/html
COPY apps/frontend/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
