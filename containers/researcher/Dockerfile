FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV PYTHONUNBUFFERED=1

RUN apt-get update && apt-get install -y \
    python3 \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash agent-e
USER agent-e
WORKDIR /workspace

EXPOSE 8005
CMD ["python3", "-m", "project.src.worker.server", "--agent-id", "agent-e", "--role", "researcher", "--port", "8005"]
