#!/bin/bash
echo "Generating \`Sunspot\` Certificate Authority Key and Certificate..."
echo ""
mkdir -p ./certificates
openssl genrsa -out src/ca/sunspot.key 2048
openssl req -x509 -new -nodes -key ./certificates/sunspot.key -sha256 -days 1825 -out ./certificates/sunspot.pem
if [ "$(uname)" == "Darwin" ]; then
  echo "Adding \`Sunspot\` Certificate Authority to your trusted login certificates..."
  security add-trusted-cert -p ssl -k ~/Library/Keychains/login.keychain-db ./certificates/sunspot.pem
  echo "Done!"
else
  echo "Please add the certificate to your trusted certificates and enable it for SSL"
fi
