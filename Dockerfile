FROM rust:alpine AS build

RUN apk add --no-cache --update nodejs npm musl-dev libressl-dev &&  \
    npm i -g tailwindcss uglify-js

WORKDIR /build
COPY . .

RUN ["tailwindcss","-i", "static/input.css","-o","static/style.css", "--minify"]
RUN ["uglifyjs","static/script.js","-o","static/script.min.js"]

RUN BUILD_SCRIPT=0 cargo build --release

FROM alpine:3.20 AS final

COPY --from=build /build/target/release/tiktok-tts /bin/tiktok-tts
EXPOSE 3000
EXPOSE 3001
CMD ["/bin/tiktok-tts"]
