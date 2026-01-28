# Install Claude Code CLI globally
curl -fsSL https://claude.ai/install.sh | bash

# Fix permissions for shared Claude config directory
sudo chown -R $(whoami) ~/.claude-shared

# Remove claude config directory if it exists before linking
rm -rf ~/.claude ~/.claude.json

# Create symbolic links for Claude config
mkdir -p ~/.claude-shared/.claude
# Create empty .claude.json file if it doesn't exist so it can be linked
[ -f ~/.claude-shared/.claude.json ] || echo '{}' > ~/.claude-shared/.claude.json

# Link the claude files on the mounted volume to the local claude files
ln -sf ~/.claude-shared/.claude ~/.claude
ln -sf ~/.claude-shared/.claude.json ~/.claude.json
