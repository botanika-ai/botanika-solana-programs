#!/bin/bash

# Botanika Solana Programs - Deployment Script

set -e

NETWORK=${1:-devnet}
PROGRAMS=${2:-all}

echo "🚀 Deploying Botanika Solana Programs to $NETWORK..."

# Set Solana network
echo "🌐 Setting network to $NETWORK..."
solana config set --url $NETWORK

# Build programs
echo "🏗️ Building programs..."
./scripts/build.sh

# Deploy programs
case $PROGRAMS in
    "staking"|"all")
        echo "📤 Deploying staking program..."
        anchor deploy --provider.cluster $NETWORK --program-name staking
        ;;
esac

case $PROGRAMS in
    "rewards"|"all")
        echo "📤 Deploying rewards program..."
        anchor deploy --provider.cluster $NETWORK --program-name rewards
        ;;
esac

case $PROGRAMS in
    "governance"|"all")
        echo "📤 Deploying governance program..."
        anchor deploy --provider.cluster $NETWORK --program-name governance
        ;;
esac

echo "✅ Deployment completed successfully!"
echo "🔍 Verify deployment with: solana program show <PROGRAM_ID>" 