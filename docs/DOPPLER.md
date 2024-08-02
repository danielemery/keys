# Doppler

[Doppler](https://www.doppler.com/) is a secrets management tool that has been
used extensively with this project.

## Development

### Running locally

```sh
doppler run -- deno run --allow-net --allow-env --allow-read=./src main.ts
```

## Kubernetes

## Prerequisites

- [Kubernetes](https://kubernetes.io/) cluster with the
  [Doppler operator](https://docs.doppler.com/docs/kubernetes-operator)
  installed
- A doppler configuration has been created for the environment you are deploying
  to (in this case we will refer to it as `stg` but it could be anything)
- A service token has been created for the doppler configuration

## Steps

1. Create a new namespace in your cluster for the test
   ```sh
   kubectl create namespace keys-test
   ```
2. Create the doppler access token secret _Replace `dp.st.dev.XXXX` with the
   service token for the target doppler environment_
   ```sh
   kubectl create secret generic keys-local-helm-test \
    --namespace doppler-operator-system \
    --from-literal=serviceToken=dp.st.dev.XXXX
   ```
3. Create the secret mapping
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
       name: keys-secret
       namespace: keys-test
       type: Opaque
   EOF
   ```
4. The doppler operator should now automatically create a secret called
   `keys-secret`. You can confirm this with the following command:
   ```sh
   kubectl describe -n keys-test secret keys-secret
   ```
5. You can now deploy the application using helm as documented in the
   [README](../README.md)
