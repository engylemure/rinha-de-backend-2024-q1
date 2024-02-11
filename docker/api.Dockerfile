FROM rust:1.76 as builder
    
# Adding system dependencies
RUN apt update && apt install --no-install-recommends -y libpq-dev build-essential cmake && rm -rf /var/lib/apt/lists/*

# Setting up working directory
ENV HOME=/opt/app

WORKDIR $HOME

COPY api/ /opt/app/api

ENV TARGET_NAME rinha
RUN cd api && cargo build --release && cd target/release && rm -rf build deps examples incremental

FROM debian:bookworm-slim

# Adding System dependencies
RUN apt update && apt install --no-install-recommends -y libpq-dev wget && rm -rf /var/lib/apt/lists/*

# Dockerize setup
ENV DOCKERIZE_VERSION v0.6.1
RUN wget https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz  --no-check-certificate \
    && tar -C /usr/local/bin -xzvf dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz \
    && rm dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz

# Setting up working directory
ENV HOME=/opt/app

WORKDIR $HOME

COPY --from=builder /opt/app/api /opt/app/api
COPY start.sh /opt/app
COPY env.tmpl /opt/app

# Application Execution 
ENTRYPOINT ["dockerize", "-template", "./env.tmpl:./api/.env"]
ENV TARGET_NAME rinha
CMD ["sh", "./start.sh"]