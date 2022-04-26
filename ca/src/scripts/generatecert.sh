#!/bin/bash

if [ "$#" -ne 3 ]
then
  echo "Usage: Must supply a domain and port and CA directory"
  exit 1
fi

DOMAIN=$1
PORT=$2

cd $3

openssl genrsa -out $DOMAIN.key 2048
openssl req -new -key $DOMAIN.key -out $DOMAIN.csr \
-subj "/C=US/ST=NC/L=Asheville/O=prox/OU=proxy/CN=$DOMAIN"

cat > $DOMAIN.ext << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names
[alt_names]
DNS.1 = $DOMAIN
EOF

openssl x509 -req -in $DOMAIN.csr -CA myCA.pem -CAkey myCA.key -CAcreateserial \
-out $DOMAIN.crt -days 825 -sha256 -extfile $DOMAIN.ext -passin pass:test

echo "$3/$DOMAIN.crt $3/$DOMAIN.key"
