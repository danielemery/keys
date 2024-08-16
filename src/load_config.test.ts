import {
  assertEquals,
  assertRejects,
} from "https://deno.land/std@0.117.0/testing/asserts.ts";
import loadConfig from "./load_config.ts";
import { ZodError } from "../deps.ts";

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
  await assertRejects(
    async () => {
      await loadConfig("./fixtures/missing-key.yaml");
    },
    ZodError,
    `"code": "invalid_type"`,
  );

  await assertRejects(
    async () => {
      await loadConfig("./fixtures/missing-field.yaml");
    },
    ZodError,
    `{
    "code": "invalid_type",
    "expected": "string",
    "received": "undefined",
    "path": [
      "ssh-keys",
      0,
      "key"
    ],
    "message": "Required"
  }`,
  );
});

Deno.test("loadConfig: must throw zod error if pgp keys are not valid", async () => {
  await assertRejects(
    async () => {
      await loadConfig("./fixtures/missing-pgp-key.yaml");
    },
    ZodError,
    `{
    "code": "invalid_type",
    "expected": "string",
    "received": "undefined",
    "path": [
      "pgp-keys",
      1,
      "key"
    ],
    "message": "Required"
  }`,
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
  });
});
