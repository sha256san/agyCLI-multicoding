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

RUN useradd -m -s /bin/bash agent-b
USER agent-b
WORKDIR /workspace

EXPOSE 8002
CMD ["python3", "-m", "project.src.worker.server", "--agent-id", "agent-b", "--role", "tester", "--port", "8002"]
