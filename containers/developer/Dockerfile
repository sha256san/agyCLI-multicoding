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

RUN useradd -m -s /bin/bash agent-a
USER agent-a
WORKDIR /workspace

EXPOSE 8001
CMD ["python3", "-m", "project.src.worker.server", "--agent-id", "agent-a", "--role", "developer", "--port", "8001"]
