FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV PYTHONUNBUFFERED=1

RUN apt-get update && apt-get install -y \
    python3 \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash agent-d
USER agent-d
WORKDIR /workspace

EXPOSE 8004
CMD ["python3", "-m", "project.src.worker.server", "--agent-id", "agent-d", "--role", "security", "--port", "8004"]
