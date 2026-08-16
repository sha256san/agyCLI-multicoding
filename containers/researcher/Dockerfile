FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV PYTHONUNBUFFERED=1

RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    git \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install agycli and mag into container
COPY target/release/agycli /usr/local/bin/agycli
COPY target/release/mag /usr/local/bin/mag
RUN chmod +x /usr/local/bin/agycli /usr/local/bin/mag

RUN useradd -m -s /bin/bash agent-e
USER agent-e
WORKDIR /workspace

EXPOSE 8005
CMD ["python3", "-m", "project.src.worker.server", "--agent-id", "agent-e", "--role", "researcher", "--port", "8005"]
