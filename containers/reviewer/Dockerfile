FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV PYTHONUNBUFFERED=1

RUN apt-get update && apt-get install -y \
    python3 \
    git \
    curl \
    flake8 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash agent-c
USER agent-c
WORKDIR /workspace

EXPOSE 8003
CMD ["python3", "-m", "project.src.worker.server", "--agent-id", "agent-c", "--role", "reviewer", "--port", "8003"]
