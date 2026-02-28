import { assert, assertEquals, assertRejects } from "@std/assert";
import loadConfig from "./load_config.ts";
import { ZodError } from "zod";

Deno.test("loadConfig: must throw error if file is not found", async () => {
  await assertRejects(
    async () => {
      await loadConfig("non-existent-file.yaml");
    },
    Deno.errors.NotFound,
    "No such file or directory",
  );
});

Deno.test("loadConfig: must throw syntax error if file is not valid yaml", async () => {
  await assertRejects(
    async () => {
      await loadConfig("./fixtures/invalid.yaml");
    },
    SyntaxError,
    "end of the stream or a document separator is expected",
  );
});

Deno.test("loadConfig: must throw zod error if ssh keys are not valid", async () => {
  const err1 = await assertRejects(
    async () => {
      await loadConfig("./fixtures/missing-key.yaml");
    },
    ZodError,
  );
  assert(err1.issues.some((i) => i.code === "invalid_type"));

  const err2 = await assertRejects(
    async () => {
      await loadConfig("./fixtures/missing-field.yaml");
    },
    ZodError,
  );
  const keyIssue = err2.issues.find((i) =>
    i.path[0] === "ssh-keys" && i.path[1] === 0 && i.path[2] === "key"
  );
  assert(keyIssue);
  assertEquals(keyIssue.code, "invalid_type");
  assertEquals(keyIssue.path, ["ssh-keys", 0, "key"]);
  assertEquals(
    keyIssue.message,
    "Invalid input: expected string, received undefined",
  );
});

Deno.test("loadConfig: must throw zod error if pgp keys are not valid", async () => {
  const err = await assertRejects(
    async () => {
      await loadConfig("./fixtures/missing-pgp-key.yaml");
    },
    ZodError,
  );
  const keyIssue = err.issues.find((i) =>
    i.path[0] === "pgp-keys" && i.path[1] === 1 && i.path[2] === "key"
  );
  assert(keyIssue);
  assertEquals(keyIssue.code, "invalid_type");
  assertEquals(keyIssue.path, ["pgp-keys", 1, "key"]);
  assertEquals(
    keyIssue.message,
    "Invalid input: expected string, received undefined",
  );
});

Deno.test("loadConfig: must load valid config with ssh keys", async () => {
  const config = await loadConfig("./fixtures/valid.yaml");
  assertEquals(config, {
    "ssh-keys": [
      {
        key: "ssh-rsa my-key-one",
        name: "key-one",
        tags: [
          "foo",
        ],
        user: "joeblogs",
      },
      {
        key: "ssh-rsa my-key-two",
        name: "pgp-yubikey",
        tags: [
          "foo",
          "bar",
        ],
        user: "joeblogs",
      },
    ],
    "pgp-keys": [],
    "known-hosts": [],
  });
});

Deno.test("loadConfig: must load valid config with pgp keys", async () => {
  const config = await loadConfig("./fixtures/valid-pgp.yaml");
  assertEquals(config, {
    "ssh-keys": [],
    "pgp-keys": [
      {
        name: "key-one",
        key: `-----BEGIN PGP PUBLIC KEY BLOCK-----

fake1
-----END PGP PUBLIC KEY BLOCK-----
`,
      },
      {
        name: "key-two",
        key: `-----BEGIN PGP PUBLIC KEY BLOCK-----

fake2
-----END PGP PUBLIC KEY BLOCK-----
`,
      },
    ],
    "known-hosts": [],
  });
});

Deno.test("loadConfig: must throw zod error if known hosts are not valid", async () => {
  const err = await assertRejects(
    async () => {
      await loadConfig("./fixtures/invalid-known-hosts.yaml");
    },
    ZodError,
  );

  const hostsIssue = err.issues.find((i) =>
    i.path[0] === "known-hosts" && i.path[1] === 0 && i.path[2] === "hosts"
  );
  assert(hostsIssue);
  assertEquals(hostsIssue.code, "invalid_type");
  assertEquals(hostsIssue.path, ["known-hosts", 0, "hosts"]);
  assertEquals(
    hostsIssue.message,
    "Invalid input: expected array, received string",
  );

  const keysIssue = err.issues.find((i) =>
    i.path[0] === "known-hosts" && i.path[1] === 0 && i.path[2] === "keys"
  );
  assert(keysIssue);
  assertEquals(keysIssue.code, "invalid_type");
  assertEquals(keysIssue.path, ["known-hosts", 0, "keys"]);
  assertEquals(
    keysIssue.message,
    "Invalid input: expected array, received undefined",
  );
});

Deno.test("loadConfig: must load valid config with known hosts", async () => {
  const config = await loadConfig("./fixtures/valid-known-hosts.yaml");
  assertEquals(config, {
    "ssh-keys": [
      {
        key: "ssh-rsa my-key-one",
        name: "key-one",
        tags: [],
        user: "joeblogs",
      },
    ],
    "pgp-keys": [],
    "known-hosts": [
      {
        name: "example",
        hosts: [
          "example.com",
        ],
        keys: [
          {
            comment: "An example key",
            key: "fake-ed25519-key",
            revoked: false,
            "cert-authority": false,
            type: "ssh-ed25519",
          },
          {
            key: "fake rsa key",
            revoked: false,
            "cert-authority": false,
            type: "ssh-rsa",
          },
        ],
      },
    ],
  });
});
