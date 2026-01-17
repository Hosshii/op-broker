#!/bin/bash

echo 'setopt multios' >> ~/.zshrc
echo 'eval "$(mise activate zsh)"' >> ~/.zshrc 
git config --global user.name Hosshii 
git config --global user.email sao_heath6147.wistre@icloud.com

SHELL=zsh pnpm setup
export PNPM_HOME="/home/vscode/.local/share/pnpm"
export PATH="$PNPM_HOME:$PATH"

pnpm set --location=global minimumReleaseAge 4320

curl -fsSL https://claude.ai/install.sh | bash

pnpm install -g @openai/codex
codex mcp add context7 -- dlx -y @upstash/context7-mcp
codex mcp add serena -- mise x uv@latest -- uvx --from git+https://github.com/oraios/serena serena start-mcp-server --context codex --project '"$PWD"'

wget -P /tmp https://github.com/dandavison/delta/releases/download/0.18.2/delta-0.18.2-x86_64-unknown-linux-gnu.tar.gz
tar -zxvf /tmp/delta-0.18.2-x86_64-unknown-linux-gnu.tar.gz -C /tmp
sudo cp /tmp/delta-0.18.2-x86_64-unknown-linux-gnu/delta /usr/local/bin

