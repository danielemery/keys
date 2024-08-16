import loadPGPKeys from "./load_pgp.ts";

import {
  assertArrayIncludes,
  assertEquals,
} from "https://deno.land/std@0.204.0/assert/mod.ts";

Deno.test("loadPGPKeys: must return PGP keys", async () => {
  const keys = await loadPGPKeys("./fixtures/valid_pgp_keys");
  assertEquals(keys.length, 2);
  assertArrayIncludes(keys, [
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
  ]);
});

Deno.test("loadPGPKeys: must ignore invalid files", async () => {
  const keys = await loadPGPKeys("./fixtures/invalid_pgp_keys");
  assertEquals(keys.length, 0);
});

Deno.test("loadPGPKeys: must return an empty set when directory does not exist", async () => {
  const keys = await loadPGPKeys("./fixtures/does_not_exist");
  assertEquals(keys.length, 0);
});
