FROM mcr.microsoft.com/devcontainers/base:debian

# https://docs.docker.com/reference/dockerfile/#example-cache-apt-packages
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
  
# Create claude config directories and set permissions
RUN mkdir -p /home/vscode/.claude && \
  chown -R vscode /home/vscode/.claude

# Create codex config directories and set permissions
RUN mkdir -p /home/vscode/.codex && \
  chown -R vscode /home/vscode/.codex
