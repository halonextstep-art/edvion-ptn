# syntax=docker/dockerfile:1

# This app has no server/api routes and its auth middleware deliberately skips SSR
# (`if (import.meta.server) return` — the token lives in localStorage, not a cookie), so it
# has no real dependency on a running Node server. `nuxt generate` + nginx serves the exact
# same build output as the SSR `node-server` preset's client bundle, without paying for a
# Node runtime at all — this is what keeps the image small.

# ─── Build ────────────────────────────────────────────────────────────────────────
FROM node:22-alpine AS build
WORKDIR /app

ARG URL_API=https://ptn.edvion.id/api
ENV NUXT_APP_BASE_URL=/
ENV NUXT_PUBLIC_API_BASE=$URL_API

COPY package.json yarn.lock ./
RUN --mount=type=cache,target=/usr/local/share/.cache/yarn \
    yarn install --frozen-lockfile

COPY . .
RUN yarn generate

# ─── Runtime ──────────────────────────────────────────────────────────────────────
FROM nginx:alpine AS runtime
COPY --from=build /app/.output/public /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
