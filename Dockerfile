# Our netlify build is with ubuntu 20.04, so that is what we use here, too.
FROM ubuntu:20.04

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential \
    curl \
    nodejs \
    && rm -rf /var/lib/apt/lists/*

ADD . .

RUN ./build.sh
