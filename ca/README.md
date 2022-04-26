# Certificate Authority

This library holds logic for dealing with a certificate authority, including:

- Parsing the private and public keys from a root certificate
- Issuing certificates for individual sites
- Generating configuration for use elsewhere

## Generating a Certificate Authority

### 1. Create a config file

```
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost:8443
```

### 2. Generate a key file

```bash
$ openssl genrsa -out myca.key 2048
```

### 3. Create x509 certs from key

```bash
$ openssl req -x509 -new -nodes -key myca.key -sha256 -days 1825 -out myca.pem
```

### 4. Create the certificate from the pem file

```bash
$ openssl x509 -in myca.pem -inform PEM -out myca.crt
```

### 5. Trust the certificate

#### macOS

```bash
$ sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ./myca.crt
```

#### Linux

Google for your distro, but here's how I did it on popOS/Ubuntu:

```bash
$ cp myca.crt /usr/local/share/ca-certificates/extra
$ sudo update-ca-certificates
```

#### Firefox/Chrome

You will need to manually import and trust the root CA from the settings menu in both Chrome and Firefox.
