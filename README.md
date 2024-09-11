# Keys

Simple repository to manage and distribute ssh keys.

To see a production implementation of this app feel free to visit
https://keys.demery.net/keys

Public keys are provided in a configuration file at application start. The
application has no persistence layer and is stateless.

[![codecov](https://codecov.io/gh/danielemery/keys/branch/main/graph/badge.svg?token=3F3EN3UY21)](https://codecov.io/gh/danielemery/keys)

## Example Usage

Typically the purpose of keys is to override the `authorized_keys` file for a
machine. This is currently done with manual curl commands, and not typically
automated with a CRON due to the risk of losing access to machines as a result
of the service being down or misconfigured.

In the future ([#24](https://github.com/danielemery/keys/issues/24)) a cli tool
will be provided to safely manage the `authorized_keys` file with guards in
place to prevent loss of access.

### Get all listed keys

```sh
curl "https://keys.demery.net/keys"
```

### Update authorized keys file

_Get keys for the `demery` user with the `oak` tag and excluding the `disabled`
tag and override the `authorized_keys` file with them_

```sh
# Consider backup first
cp ~/.ssh/authorized_keys ~/.ssh/authorized_keys.`date '+%Y-%m-%d__%H_%M_%S'`.backup
# Override file with the matching keys
curl "https://keys.demery.net/keys?user=demery&allOf=oak&noneOf=disabled" > ~/.ssh/authorized_keys
# Check that they keys were updated with what you expected
cat ~/.ssh/authorized_keys
```

### Update known hosts file

_Replaces the `known_hosts` file with the hosts in your keys instance_

```sh
# Consider backup first
cp ~/.ssh/known_hosts ~/.ssh/known_hosts.`date '+%Y-%m-%d__%H_%M_%S'`.backup
# Override file with the hosts from the keys instance
curl http://localhost:8000/known_hosts > ~/.ssh/known_hosts
```

## Running / Installation

### Configuration File

Regardless of the method of deployment, the `keys` application requires a config
yaml file containing the list of keys to be served. An example file can be found
in `./examples/keys-config.yaml`.

The config file contains three main sections:

- `ssh-keys`: A list of public ssh keys with the following fields:
  - name: The name of the key (this will be used as the `@host` in the
    `authorized_keys` file)
  - key: The public key itself
  - user: The user that the key should be associated with (this will be used as
    the `user@host` in the `authorized_keys` file)
  - tags: Optionally a list of tags that can be used to filter the keys
- `pgp-keys`: A list of public pgp keys with the following fields:
  - name: The name of the key (this will be used in the route and as the
    filename if you download the key)
  - key: The public key itself
- `known-hosts`: A list of known hosts with the following fields:
  - name: Optional name for the entry, it's not used in the `known_hosts` file
    and is just for your records
  - hosts: A list of hostnames or IPs that the key(s) should be associated with
  - keys: A list of known keys that should be associated with the host, with the
    following fields:
    - type: The type of key (eg `ssh-rsa`)
    - key: The public key itself
    - comment: Optional comment for the entry (will be appended to the key in
      the `known_hosts` file)
    - revoked: Optional boolean to indicate that the key should be considered
      revoked (adds the @revoked marker in the known hosts file)
    - cert-authority: Optional boolean to indicate that the key is a certificate
      authority (adds the @cert-authority marker in the known hosts file)

### Helm

#### Secret Creation

The recommended method of deploying the `keys` application is using the official
helm chart.

Before deployment, it's expected to have a secret created within the target
namespace that contains the environment variable configuration for the
application. This secret is expected to be named `keys-secret` but can be
overriden by setting the `secrets.secretName` value in the helm chart.

The secret could be created using doppler (see
[Doppler docs](./docs/DOPPLER.md)) or another secrets solution. Or by simply
creating the secret file manually (recommended for one-off tests).

Manual secret creation:

1. Create a new namespace in your cluster for the test
   ```sh
   kubectl create namespace keys-test
   ```
2. Create the secret file
   ```sh
   kubectl create secret generic keys-secret \
     --from-literal=DOPPLER_ENVIRONMENT=local-helm-test
   ```

#### Chart Repository

The following assumes you have created the namespace and secrets as
[described above](#secret-creation).

```sh
helm repo add keys https://helm.demery.net
helm repo update
helm install -n keys-test --set version=v2.0.0 --set configFile.content="$(cat ./examples/keys-config.yaml)" keys demery/keys
```

### Docker

The `keys` application is packaged in docker and the image can be found in the
[Github registry](https://ghcr.io/danielemery/keys).

_The inner container port is always **8000** unless overriden with the `PORT`
environment variable. Note that port **80** cannot be used due to limitations of
the way Deno handles permissions._

```sh
docker run \
  -p 8000:8000 \
  -v $(pwd)/examples/keys-config.yaml:/config.yaml \
  -e DOPPLER_ENVIRONMENT=local-docker \
  -e KEYS_VERSION=local-docker \
  ghcr.io/danielemery/keys:latest
```

#### Docker Compose

When using docker instead of the helm chart it's recommended to use a
docker-compose file.

An example is provided in `examples/docker-compose.yaml` and can be run with the
following command:

```sh
docker compose -f examples/docker-compose.yaml up
```

## Development

### Running locally

Make a copy of the `.env.example` file and rename it to `.env`. Review it's
contents and make any necessary changes.

```sh
cp .env.example .env
```

Then run the following command to start the server:

```sh
deno run --env --allow-net --allow-env --allow-read main.ts
```

### Run tests

```sh
deno test --allow-read
```

### Local Helm Chart

It can be useful to run the chart directly from the repository for testing or
for using the chart with a version that has not yet been published.

1. Install the chart
   ```sh
   # Replace the version with the desired version. It will need to be a version that exists in the Github registry.
   helm install -n keys-test --set version=v2.0.0 --set configFile.content="$(cat ./examples/keys-config.yaml)" keys ./helm
   ```
2. Port forward to the service for testing
   ```sh
   kubectl -n keys-test port-forward svc/keys-svc 8000:80
   ```

### Local Docker Build

It can also be useful to build the docker image locally for testing of
`Dockerfile` changes. This can be done with the following commands:

```sh
docker build -t keys:local .
docker run \
  -p 8000:8000 \
  -v $(pwd)/examples/keys-config.yaml:/config.yaml \
  -e DOPPLER_ENVIRONMENT=local-docker \
  -e KEYS_VERSION=local-docker \
  keys:local
```

### Branch / Release strategy

- Tags should follow [semantic versioning](https://semver.org/)
- Most PRs should be made to `main` branch
  - Patch and minor Tags will be created on the `main` branch eg
    `v1.3.0`/`v1.3.1`
  - Usually a tag and release will be performned after each merge to `main`, but
    sometimes multiple PRs can be merged before a tag is created
- Breaking change PRs should be made to the `next` branch
  - Release candidate tags will be created on the `next` branch for the next
    breaking change version (eg `v2.0.0-rc.1`)
  - Once the release candidate has been validated and is ready to be released,
    the `next` branch will be merged into `main` and a new tag will be created
    on `main` (eg `v2.0.0`)
  - The `next` branch should only be merged to `main` using the fast-forward
    merge strategy
  - Each breaking change version should have it's progress tracked using a
    milestone on GitHub
- Breaking change PRs should always be labelled as such (and the future a rule
  will be created not allow them to be directly merged into `main`)
