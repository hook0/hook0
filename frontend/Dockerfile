ARG NODE_VERSION=20
ARG CADDY_VERSION=2.9

FROM node:${NODE_VERSION} AS build
WORKDIR /app
ARG API_ENDPOINT http://localhost:8081/api/v1
COPY frontend/ ./
RUN --mount=type=bind,source=mediakit,target=/mediakit \
    rm -rf -- dist/ && npm ci && npm run build

FROM caddy:${CADDY_VERSION}
RUN sed -i '/file_server/i try_files {path} /index.html' /etc/caddy/Caddyfile
RUN apk --no-cache add curl
COPY --from=build /app/dist /usr/share/caddy/

EXPOSE 80
