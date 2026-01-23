ARG NODE_VERSION=24
ARG CADDY_VERSION=2.10

FROM node:${NODE_VERSION} AS build
WORKDIR /app
ENV API_ENDPOINT=API_ENDPOINT_PLACEHOLDER
COPY frontend/ ./
RUN --mount=type=bind,source=mediakit,target=/mediakit \
    rm -rf -- dist/ && npm ci && npm run build

FROM caddy:${CADDY_VERSION}
RUN sed -i '/file_server/i try_files {path} /index.html' /etc/caddy/Caddyfile
RUN apk --no-cache add curl
COPY --from=build /app/dist /usr/share/caddy/

COPY frontend/scripts/docker-entrypoint.sh /docker-entrypoint.sh

EXPOSE 80
CMD ["/docker-entrypoint.sh"]
