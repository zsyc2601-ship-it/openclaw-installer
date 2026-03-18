FROM rust:1.82-bookworm AS builder

# System deps for Tauri Linux build — retry on failure
RUN --mount=type=cache,target=/var/cache/apt \
    apt-get update && \
    apt-get install -y --fix-missing \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libssl-dev \
    wget \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 22
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g pnpm@9

# Install tauri-cli
RUN cargo install tauri-cli --version "^2"

WORKDIR /app

# Copy manifests first for cache
COPY package.json pnpm-workspace.yaml tsconfig.json ./
COPY apps/installer/package.json apps/installer/

# Install frontend deps
RUN cd apps/installer && pnpm install

# Copy source
COPY apps/installer/ apps/installer/
COPY scripts/ scripts/

# Download Node.js archives
RUN bash scripts/download-node.sh

# Build frontend + Tauri
RUN cd apps/installer && pnpm build && cargo tauri build

# Output stage
FROM scratch AS output
COPY --from=builder /app/apps/installer/src-tauri/target/release/bundle/ /bundle/
