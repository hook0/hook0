ARG NODE_VERSION=18

FROM node:${NODE_VERSION} as build
WORKDIR /app
ENV API_ENDPOINT http://localhost:8081/api/v1
COPY frontend/ ./
RUN --mount=type=bind,source=mediakit,target=/mediakit \
    rm -rf -- dist/ && npm ci && npm run build

FROM nginx
RUN rm -v /etc/nginx/nginx.conf
COPY self-hosted/docker/frontend/nginx.conf /etc/nginx/
COPY --from=build /app/dist /var/www/

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
