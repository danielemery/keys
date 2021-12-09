# Keys

Simple repository to manage and distribute my public keys

## Example curl Calls

### Get all listed keys

```sh
curl https://keys.demery.com.au/api
```

### Get keys matching single tag (server)

```sh
curl https://keys.demery.com.au/api?oneOf=server
```

### Get keys matching multiple tags (server OR git)

```sh
curl https://keys.demery.com.au/api?oneOf=server&oneOf=git
```

## Usage

### Running locally

```sh
deno run --allow-net main.ts
```

### Running with docker

```sh
docker build -t keys:latest .
docker run -p 8000:8000 keys:latest
```
