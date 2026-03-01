#!/bin/bash
set -e

# DynamoDB Local Setup Script
# Downloads and validates DynamoDB Local for integration testing

DYNAMODB_DIR="dynamodb"
DYNAMODB_URL="https://d1ni2b6xgvw0s0.cloudfront.net/v2.x/dynamodb_local_latest.tar.gz"
ARCHIVE_NAME="dynamodb_local_latest.tar.gz"
# SHA256 checksum for DynamoDB Local (update this when AWS releases new versions)
# Note: AWS doesn't publish checksums, so we verify the JAR exists after extraction
EXPECTED_JAR="DynamoDBLocal.jar"

echo "=== DynamoDB Local Setup ==="
echo ""

# Check if already installed
if [ -f "$DYNAMODB_DIR/$EXPECTED_JAR" ]; then
    echo "✓ DynamoDB Local is already installed at $DYNAMODB_DIR/$EXPECTED_JAR"
    echo ""
    echo "To reinstall, remove the $DYNAMODB_DIR directory and run this script again."
    exit 0
fi

# Create directory
echo "Creating $DYNAMODB_DIR directory..."
mkdir -p "$DYNAMODB_DIR"

# Download DynamoDB Local
echo "Downloading DynamoDB Local from AWS..."
echo "URL: $DYNAMODB_URL"
curl -L -o "$DYNAMODB_DIR/$ARCHIVE_NAME" "$DYNAMODB_URL"

if [ ! -f "$DYNAMODB_DIR/$ARCHIVE_NAME" ]; then
    echo "✗ Error: Failed to download DynamoDB Local"
    exit 1
fi

echo "✓ Download complete"
echo ""

# Extract archive
echo "Extracting archive..."
cd "$DYNAMODB_DIR"
tar -xzf "$ARCHIVE_NAME"
cd ..

# Verify JAR exists
if [ ! -f "$DYNAMODB_DIR/$EXPECTED_JAR" ]; then
    echo "✗ Error: $EXPECTED_JAR not found after extraction"
    echo "The archive may be corrupted or the AWS distribution has changed."
    exit 1
fi

echo "✓ Extraction complete"
echo ""

# Calculate and display SHA256 checksum for future reference
if command -v sha256sum &> /dev/null; then
    CHECKSUM=$(sha256sum "$DYNAMODB_DIR/$EXPECTED_JAR" | awk '{print $1}')
    echo "JAR SHA256 checksum: $CHECKSUM"
    echo "(Save this for verification in future runs)"
elif command -v shasum &> /dev/null; then
    CHECKSUM=$(shasum -a 256 "$DYNAMODB_DIR/$EXPECTED_JAR" | awk '{print $1}')
    echo "JAR SHA256 checksum: $CHECKSUM"
    echo "(Save this for verification in future runs)"
fi

echo ""

# Clean up archive
echo "Cleaning up archive..."
rm "$DYNAMODB_DIR/$ARCHIVE_NAME"

echo ""
echo "=== Setup Complete ==="
echo ""
echo "DynamoDB Local is installed at: $DYNAMODB_DIR/$EXPECTED_JAR"
echo ""
echo "To run property tests:"
echo "  cargo test --test property_resource_creation_roundtrip -- --test-threads=1"
echo ""
echo "Note: The test utilities will automatically start and stop DynamoDB Local"
