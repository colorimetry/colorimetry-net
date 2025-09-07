FROM ubuntu:24.04

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential \
    curl \
    python3 \
    && rm -rf /var/lib/apt/lists/*

ADD . .

# This builds all output as static files into the `dist` directory.
RUN ./build.sh

EXPOSE 8000/tcp

WORKDIR /dist

# Run the simple webserver
CMD ["/usr/bin/python3", "-m", "http.server", "8000", "--bind", "0.0.0.0"]
