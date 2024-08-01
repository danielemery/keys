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
doppler run -- deno run --allow-net --allow-env --allow-read main.ts
```

### Run tests

```sh
deno test
```

## Running

### Docker

#### Github Registry

```sh
# TODO
```

#### Local build & run

```sh
docker build -t keys:latest .

# Run, exposing port 8000 (inner port is always 80)
docker run -p 8000:80 keys:latest
```

### Helm

### Chart Repository

```sh
# TODO
```

### Local Chart

It can be useful to run the chart directly from the repository for testing or
for using the chart with a version that has not yet been published.

This can be done using doppler for secrets management (like in production -
recommended for long-lived test environements). Or by simply creating the secret
file manually (recommended for one-off tests).

#### Doppler

##### Prerequisites

- Kubernetes cluster with the doppler operator installed

##### Steps

1. Make a doppler config branch. We will refer to it as
   `staging_local-helm-test` in this example. Also make a service token for the
   branch and have it ready for the later steps.
2. Create a new namespace in your cluster for the test
   ```sh
   kubectl create namespace keys-test
   ```
3. Create the doppler access token secret
   ```sh
   kubectl create secret generic keys-local-helm-test \
    --namespace doppler-operator-system \
    --from-literal=serviceToken=dp.st.dev.XXXX
   ```
4. Create the secret mapping
   ```sh
   kubectl apply -f - <<EOF
   apiVersion: secrets.doppler.com/v1alpha1
   kind: DopplerSecret
   metadata:
     name: local-helm-test
     namespace: doppler-operator-system
   spec:
     tokenSecret:
       name: keys-local-helm-test
     managedSecret:
       name: doppler-keys-secret
       namespace: keys-test
       type: Opaque
   EOF
   ```
5. Install the chart
   ```sh
   # Replace the version with the desired version. It will need to be a version that exists in the Github registry.
   helm install -n keys-test --set version=v2.0.0-config-file.0 keys ./helm
   ```
6. Port forward to the service for testing
   ```sh
   kubectl -n keys-test port-forward svc/keys-svc 8000:80
   ```
