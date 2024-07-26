# Keys

Simple repository to manage and distribute my public ssh keys.

Public keys are stored statically in the repository and hosted at
https://keys.demery.net

[![codecov](https://codecov.io/gh/danielemery/keys/branch/main/graph/badge.svg?token=3F3EN3UY21)](https://codecov.io/gh/danielemery/keys)

## Example Usage

### Get all listed keys

```sh
curl "https://keys.demery.net/api"
```

### Get keys for demery account on thunderbird and override authorized_keys file with them

```sh
# Consider backup first
cp ~/.ssh/authorized_keys ~/.ssh/authorized_keys.`date '+%Y-%m-%d__%H_%M_%S'`.backup
# Override file with that remote goodness
curl "https://keys.demery.net/api?allOf=demery&allOf=thunderbird&noneOf=disabled" > ~/.ssh/authorized_keys
```

## Development

### Running locally

```sh
doppler run -- deno run --allow-net --allow-env --allow-read=./src main.ts
```

### Run tests

```sh
deno test
```

### Running with docker

## Local build

```sh
docker build -t keys:latest .

# Run, exposing port 8000 (inner port is always 80)
docker run -p 8000:80 keys:latest
```
