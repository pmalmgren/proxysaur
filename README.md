![](https://img.shields.io/github/checks-status/pmalmgren/proxysaur/main) ![](https://img.shields.io/github/v/release/pmalmgren/proxysaur) ![](https://img.shields.io/github/release-date/pmalmgren/proxysaur)  ![](https://img.shields.io/github/last-commit/pmalmgren/proxysaur) 

[Docs Page](https://proxysaur.us)

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

Pay attention to the commands listed in the output underneath "To trust this certificate." For example, on macOS you'd run the line that looks like:

```bash
security add-trusted-cert -d -r trustRoot -k $HOME/Library/Keychains/login.keychain CA_LOC/myca.crt
```

### Test it out

In a separate terminal, run:

```bash
$ curl -x "http://localhost:9999" "https://proxysaur.us/test"
```

You should observe that the request has been rewritten :)

## Documentation

Head over to the [docs page](https://proxysaur.us/) to learn about how to use proxysaur to debug HTTP applications.
