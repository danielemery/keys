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
