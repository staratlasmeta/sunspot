# Sunspot Proxy
[![Crates.io](https://img.shields.io/crates/v/sunspot)](https://crates.io/crates/sunspot) 
![Crates.io](https://img.shields.io/crates/l/sunspot) 
![Crates.io](https://img.shields.io/crates/d/sunspot)

Sunspot is a proxy server that enables custom RPC urls and custom token metadata lists for
the [Solflare](https://solflare.com) Solana wallet.

## Installation

```bash
cargo install sunspot
```

## Setting up Local Certificate Authority

### Generating Certificates

#### From repo root

```bash
chmod +x generate-cert.sh
./generate-cert.sh
```

#### From anywhere

```bash
mkdir -p $HOME/.config/sunspot/certs
openssl genrsa -out $HOME/.config/sunspot/certs/sunspot.key 2048
openssl req -x509 -new -nodes -key $HOME/.config/sunspot/certs/sunspot.key -sha256 -days 1825 -out $HOME/.config/sunspot/certs/sunspot.pem
```

### Adding CA to Chrome

1. Go to <chrome://settings/certificates> in your Chrome browser
2. Go to the `Authorities` tab and then `Import`
3. Select the `sunspot.pem` file from the `certs` directory
4. Select `Trust this certificate for identifying websites` and click `OK`

### Adding CA to Firefox

1. Go to <about:preferences#privacy> in your Firefox browser and scroll down to `Certificates`
2. Click `View Certificates`, go to the `Authorities` tab, and then click `Import`
3. Select the `sunspot.pem` file from the `certs` directory
4. Select `Trust this CA to identify websites.` and click `OK`

## Setting up SwitchyOmega Proxy


1. Install from the [Chrome Web Store](https://chrome.google.com/webstore/detail/padekgcemlokbadohgkifijomclgjgif) or
   [Firefox Add-ons](https://addons.mozilla.org/en-US/firefox/addon/switchyomega/)
2. In the SwitchyOmega options, go to Import/Export and click `Restore from file`
3. Select the [`OmegaOptions.bak`](./switchy-omega-proxy/OmegaOptions.bak) file from the `switchy-omega-proxy` directory
4. Click `Apply Changes` and enable the `auto switch` option through the extension icon

## Usage

Using the Testnet RPC option for Solflare is recommended. This will cause Solflare to route transactions through Testnet nodes,
so if an API changes unexpectedly, real transactions won't be routed to Mainnet-Beta. Testnet also disables some features, 
which is what we want since Sunspot can't support everything (yet)! 

```bash
sunspot --help
```

By default, Sunspot will try to pull the `sunspot.pem` and `sunspot.key` files from `$HOME/.config/sunspot/certs`. 

```bash
sunspot http://localhost:8899
```

If you want to use a custom certificate location, you can use the `--certificate` (`-c`) and `--key` (`-k`) flags.

```bash
sunspot -k ./certs/sunspot.key -c ./certs/sunspot.pem http://localhost:8899
```

### Using a Custom Token List File

Sunspot allows you to provide a custom token-list JSON file, which is used to add custom names, symbols, and imageURIs
to tokens in both the wallet view and during simulations.


```json
{
  "<Token Mint String>": {
    "name": "<Token Name>",
    "symbol": "<Token Symbol>",
    "imageUri": "<Token Image URI>"
  },
  // USD Coin Example
  "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v": {
    "name": "USD Coin",
    "symbol": "USDC",
    "imageUri": "https://assets.coingecko.com/coins/images/6319/large/USD_Coin_icon.png?1547042389"
  }
}
```

Plop the token list file in `$HOME/.config/sunspot/tokens/<TOKEN_NAME>.json`. 
You can pass this file to Sunspot using the `--token-list` (`-t`) flag, providing the <TOKEN_NAME> of the file. 

```bash
sunspot -t <TOKEN_NAME> http://localhost:8899
```

## Thanks

Special thanks to the [Hudsucker](https://github.com/omjadas/hudsucker) crate for the MITM HTTP/S proxy implementation