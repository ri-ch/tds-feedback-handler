# Base: Microsoft's universal devcontainer image (Debian-based, includes common tools)
FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=${USER_UID}

# ── System dependencies ────────────────────────────────────────────────────────
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Build essentials
    build-essential \
    pkg-config \
    libssl-dev \
    # Utilities
    curl \
    wget \
    git \
    unzip \
    gnupg \
    jq \
    ripgrep \
    fd-find \
    ca-certificates \
    gnupg \
    lsb-release \
    # Python (required for AWS CDK and some Rust tooling)
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

# ── Node.js 20 LTS (required for AWS CDK and Claude Code) ─────────────────────
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# ── GitHub CLI ────────────────────────────────────────────────────────────────
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
        -o /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
        > /etc/apt/sources.list.d/github-cli.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends gh \
    && rm -rf /var/lib/apt/lists/*

# ── AWS CDK + Claude Code (global npm installs) ───────────────────────────────
RUN npm install -g \
    aws-cdk@2.170.0 \
    @anthropic-ai/claude-code@latest \
    typescript@5.7.3 \
    ts-node@10.9.2

# Add AWS GPG key for verifying AWS CLI downloads
ADD aws-gpg-key.pub /tmp/aws-gpg-key.pub

# ── AWS CLI v2 ─────────────────────────────────────────────────────────────────
RUN ARCH=$(dpkg --print-architecture) \
    && if [ "$ARCH" = "amd64" ]; then AWS_ARCH="x86_64"; else AWS_ARCH="aarch64"; fi \
    && echo "Downloading AWS CLI v2 for architecture: $AWS_ARCH" \
    && curl -vfsSL "https://awscli.amazonaws.com/awscli-exe-linux-${AWS_ARCH}.zip" -o /tmp/awscliv2.zip \
    && gpg --import /tmp/aws-gpg-key.pub \
    && curl "https://awscli.amazonaws.com/awscli-exe-linux-${AWS_ARCH}.zip.sig" -o /tmp/awscliv2.sig \
    && gpg --verify /tmp/awscliv2.sig /tmp/awscliv2.zip \
    && unzip -q /tmp/awscliv2.zip -d /tmp \
    && /tmp/aws/install \
    && rm -rf /tmp/aws /tmp/awscliv2.zip /tmp/awscliv2.sig

# ── Rust (installed as the vscode user) ───────────────────────────────────────
USER ${USERNAME}

ENV RUSTUP_HOME=/home/${USERNAME}/.rustup
ENV CARGO_HOME=/home/${USERNAME}/.cargo
ENV PATH=/home/${USERNAME}/.cargo/bin:${PATH}

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile default \
    && rustup component add \
        rustfmt \
        clippy \
        rust-src \
        rust-analyzer

# Common Rust tools
# RUN cargo install cargo-watch cargo-edit cargo-audit

USER root

# ── Workspace ─────────────────────────────────────────────────────────────────
WORKDIR /workspace
