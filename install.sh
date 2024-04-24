#!/bin/sh

# Check if jq is installed
if ! command -v jq >/dev/null 2>&1; then
    echo "jq is not installed. Attempting to install..."

    # Detect the package manager and install jq
    if command -v apt-get >/dev/null 2>&1; then
        sudo apt-get update && sudo apt-get install -y jq
    elif command -v yum >/dev/null 2>&1; then
        sudo yum install -y jq
    elif command -v brew >/dev/null 2>&1; then
        brew install jq
    else
        echo "No known package manager found. Please install jq manually."
        exit 1
    fi
else
    echo "jq is already installed."
fi

# Get the latest release from the GitHub API
latest_release=$(curl -s https://api.github.com/repos/aesthetic0001/totp-cli/releases/latest)

#get the relevant asset, would be totp-macos, totp-linux, totp-windows.exe
OS=$(uname -s)
if [ "$OS" == "Darwin" ]; then
    asset_name="totp-macos"
elif [ "$OS" == "Linux" ]; then
    asset_name="totp-linux"
elif [ "$OS" == "Windows_NT" ]; then
    asset_name="totp-windows.exe"
else
    echo "Unsupported OS"
    exit 1
fi

# Get the download URL for the asset
download_url=$(echo "$latest_release" | jq -r ".assets[] | select(.name == \"$asset_name\") | .browser_download_url")

# Download the asset
sudo curl -L -o /usr/local/bin/totp $download_url

# Make the asset executable
sudo chmod +x /usr/local/bin/totp

# Check if the asset was installed successfully
if [ $? -eq 0 ]; then
    echo "totp-cli was installed successfully"
else
    echo "totp-cli could not be installed"
fi