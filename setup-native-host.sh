#!/bin/bash

echo "🔧 Feitian SK Manager - Native Host Setup"
echo "========================================"
echo ""

# Get absolute path to project
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY_PATH="$PROJECT_DIR/native/target/release/feitian-sk-manager-native"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Native host binary not found at:"
    echo "   $BINARY_PATH"
    exit 1
fi

# Make it executable
chmod +x "$BINARY_PATH"

echo "Binary found at:"
echo "   $BINARY_PATH"
echo ""

# Get extension ID
echo "Enter your Chrome Extension ID:"
echo "   (Go to chrome://extensions and copy the ID)"
echo "   Example: abcdefghijklmnopqrstuvwxyz123456"
echo ""
read -p "Extension ID: " EXTENSION_ID

if [ -z "$EXTENSION_ID" ]; then
    echo "Error: Extension ID cannot be empty"
    exit 1
fi

# Remove any accidental spaces
EXTENSION_ID=$(echo "$EXTENSION_ID" | tr -d '[:space:]')

# Create manifest directory
MANIFEST_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
mkdir -p "$MANIFEST_DIR"

# Create the manifest file
MANIFEST_FILE="$MANIFEST_DIR/com.feitian.sk_manager.json"

cat > "$MANIFEST_FILE" << EOF
{
  "name": "com.feitian.sk_manager",
  "description": "Feitian Security Key Manager Native Host",
  "path": "$BINARY_PATH",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://$EXTENSION_ID/"
  ]
}
EOF

echo ""
echo "Native host manifest created!"
echo ""
echo "Manifest location:"
echo "   $MANIFEST_FILE"
echo ""
echo "Manifest contents:"
echo "────────────────────────────────────────"
cat "$MANIFEST_FILE"
echo "────────────────────────────────────────"
echo ""
echo "Next steps:"
echo ""
echo "   1. Completely QUIT Chrome:"
echo "      Press Cmd+Q (or Chrome menu → Quit Google Chrome)"
echo ""
echo "   2. Restart Chrome"
echo ""
echo "   3. Go to: chrome://extensions"
echo "      Find 'Feitian SK Manager' and click the reload icon"
echo ""
echo "   4. Start the web UI:"
echo "      cd web && npm run dev"
echo ""
echo "   5. Open: http://localhost:5173"
echo ""
echo "✨ You should see both connection statuses turn GREEN!"
echo ""