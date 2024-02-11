FROM rust:1.76

# Dockerize setup
ENV DOCKERIZE_VERSION v0.6.1
RUN wget https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz \
    && tar -C /usr/local/bin -xzvf dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz \
    && rm dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz
    
# Adding system dependencies
RUN apt update && apt install --no-install-recommends -y libpq-dev build-essential cmake && rm -rf /var/lib/apt/lists/*

# Setting up working directory
ENV HOME=/opt/app

WORKDIR $HOME

COPY api/ /opt/app/api
COPY start.sh /opt/app
COPY env.tmpl /opt/app

# Application Setup
ENTRYPOINT ["dockerize", "-template", "./env.tmpl:./api/.env"]

ENV TARGET_NAME rinha
RUN cd api && cargo build --release && cd target/release && rm -rf build deps examples incremental

# Application Execution

CMD ["sh", "./start.sh"]