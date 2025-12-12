#!/bin/bash

# OpenGRC UI - Install and Build Script
set -e

echo "================================"
echo "OpenGRC UI - Setup & Build"
echo "================================"
echo ""

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "Error: Node.js is not installed. Please install Node.js 18+ first."
    exit 1
fi

echo "Node version: $(node --version)"
echo "NPM version: $(npm --version)"
echo ""

# Install dependencies
echo "Installing dependencies..."
npm install
echo ""

# Create .env.local if it doesn't exist
if [ ! -f .env.local ]; then
    echo "Creating .env.local from .env.example..."
    cp .env.example .env.local
    echo "Please update .env.local with your API URL if needed."
    echo ""
fi

# Run linter
echo "Running ESLint..."
npm run lint
echo ""

# Build the project
echo "Building the project..."
npm run build
echo ""

echo "================================"
echo "Setup complete!"
echo "================================"
echo ""
echo "To start the development server:"
echo "  npm run dev"
echo ""
echo "To view the production build:"
echo "  npx serve out"
echo ""
