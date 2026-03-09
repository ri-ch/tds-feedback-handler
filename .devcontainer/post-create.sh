#!/usr/bin/env bash
# .devcontainer/post-create.sh
# Runs once after the container is first created.
set -euo pipefail

echo "── Post-create setup ──────────────────────────────────────────────────────"

# ── Rust: install toolchain declared in rust-toolchain.toml ──────────────────
echo "› Installing Rust toolchain..."
rustup show

# ── CDK TypeScript project bootstrap (if cdk project exists) ──────────────────
if [ -f "package.json" ]; then
  echo "› Installing Node dependencies..."
  npm ci
fi

# ── Fix permissions on mounted .claude directory ──────────────────────────────
# (Needed when host UID doesn't match container UID)
if [ -d "/home/vscode/.claude" ]; then
  sudo chown -R vscode:vscode /home/vscode/.claude 2>/dev/null || true
fi
if [ -f "/home/vscode/.claude.json" ]; then
  sudo chown vscode:vscode /home/vscode/.claude.json 2>/dev/null || true
fi

# ── Verify key tools ──────────────────────────────────────────────────────────
echo ""
echo "── Tool versions ──────────────────────────────────────────────────────────"
echo "  rustc   : $(rustc --version)"
echo "  cargo   : $(cargo --version)"
echo "  node    : $(node --version)"
echo "  npm     : $(npm --version)"
echo "  cdk     : $(cdk --version)"
echo "  aws     : $(aws --version 2>&1)"
echo "  claude  : $(claude --version 2>/dev/null || echo 'run: claude to authenticate')"
echo ""
echo "── Setup complete ─────────────────────────────────────────────────────────"
echo "  Next steps:"
echo "    1. Authenticate Claude Code:  claude"
echo "    2. Configure AWS profile:     aws configure  (or set env vars)"
echo "    3. Bootstrap CDK environment: cdk bootstrap"
