#!/bin/bash
echo "Generating \`Sunspot\` Certificate Authority Key and Certificate..."
echo ""
mkdir -p ./certs
openssl genrsa -out ./certs/sunspot.key 2048
openssl req -x509 -new -nodes -key ./certs/sunspot.key -sha256 -days 1825 -out ./certs/sunspot.pem
if [ "$(uname)" == "Darwin" ]; then
  echo "Adding \`Sunspot\` Certificate Authority to your trusted login certificates..."
  security add-trusted-cert -p ssl -k ~/Library/Keychains/login.keychain-db ./certs/sunspot.pem
  echo "Done!"
elif [ "$(uname)" == "Linux" ] && [ -d "/usr/local/share/ca-certificates/" ]; then
  echo "Copying \`Sunspot\` Certificate Authority to \`/usr/local/share/ca-certificates/sunspot.crt\` and updating trusted certificates..."
  sudo cp ./certs/sunspot.pem /usr/local/share/ca-certificates/sunspot.crt
  sudo update-ca-certificates
  echo "Done!"
else
  echo "Please add \`./certs/sunspot.pem\` to your trusted certificates."
fi
