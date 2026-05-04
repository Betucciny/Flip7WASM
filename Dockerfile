# ---------- Build stage ----------
FROM node:20 AS builder

WORKDIR /app

# Install system deps + Rust
RUN apt-get update && apt-get install -y curl build-essential \
  && curl https://sh.rustup.rs -sSf | sh -s -- -y

# Add Cargo to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Install wasm-pack
RUN npm install -g wasm-pack

# Copy project files
COPY . .

# ---------- Build Rust WASM ----------
WORKDIR /app/rust/engine
RUN wasm-pack build --release --target web

# ---------- Build frontend ----------
WORKDIR /app/frontend
RUN npm install
RUN npm run build

# ---------- Runtime stage ----------
FROM nginx:alpine

# Copy built frontend
COPY --from=builder /app/frontend/dist /usr/share/nginx/html

# SPA routing support
RUN rm /etc/nginx/conf.d/default.conf
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
