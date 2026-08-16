FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV PYTHONUNBUFFERED=1

RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    git \
    curl \
    build-essential \
    dbus-x11 \
    libsecret-1-0 \
    gnome-keyring \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install agycli, mag, and entrypoint into container
COPY target/release/agycli /usr/local/bin/agycli
COPY target/release/mag /usr/local/bin/mag
COPY containers/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/agycli /usr/local/bin/mag /usr/local/bin/entrypoint.sh

RUN useradd -m -s /bin/bash agent-c
USER agent-c
WORKDIR /workspace

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
EXPOSE 8003
CMD ["python3", "-m", "project.src.worker.server", "--agent-id", "agent-c", "--role", "reviewer", "--port", "8003"]
