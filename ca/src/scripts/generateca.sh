#!/bin/bash

if [ "$#" -ne 1 ]
then
  echo "Usage: Must supply a domain and port and CA directory"
  exit 1
fi

cd $1

openssl genrsa -out myca.key 2048
openssl req -x509 -new -nodes -key myca.key -sha256 -days 1 -out myca.pem -subj "/C=US/ST=NC/L=Asheville/O=prox/OU=proxy"
openssl x509 -in myca.pem -inform PEM -out myca.crt