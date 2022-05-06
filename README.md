# proxysaur

A HTTP proxy debugging tool.

## Installation

Navigate to the [releases](https://github.com/pmalmgren/proxysaur/releases) page and download a compiled release for your platform.

## Getting started

Once you've downloaded the release for your platform, run the following command to launch a debugging HTTP(S) proxy:

```bash
$ proxysaur http
```

### Trust the CA

Pay attention to the commands listed in the output underneath "To trust this certificate." For example, on macOS you'd run:

```bash
$ security add-trusted-cert -d -r trustRoot -k $HOME/Library/Keychains/login.keychain CA_LOC/myca.crt
```

Where `CA_LOC` is the location of the certificate in the output.

### Test it out

In a separate terminal, run:

```bash
$ curl -x "http://localhost:9999" "https://proxysaur.us/test"
```

You should observe that the request has been rewritten :)

## Documentation

Head over to the [docs page](https://proxysaur.us/) to learn about how to use proxysaur to debug HTTP applications.