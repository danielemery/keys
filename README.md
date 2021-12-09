# Keys

Simple repository to manage and distribute my public ssh keys.

Public keys are stored statically in the repository and hosted at https://keys.demery.com.au

## Example Usage

### Get all listed keys

```sh
curl "https://keys.demery.com.au/api"
```

### Get keys for demery account on thunderbird and override authorized_keys file with them

```sh
# Consider backup first
cp ~/.ssh/authorized_keys ~/.ssh/authorized_keys.backup
# Override file with that remote goodness
curl "https://keys.demery.com.au/api?allOf=demery&allOf=thunderbird&noneOf=disabled" > ~/.ssh/authorized_keys
```

## Development

### Running locally

```sh
deno run --allow-net main.ts
```

### Run tests

```sh
deno test
```

### Running with docker

## Local build

```sh
docker build -t keys:latest .
docker run -p 8000:8000 keys:latest
```
