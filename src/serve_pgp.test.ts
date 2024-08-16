import {
  getPGPTarget,
  isValidPGPExtension,
  servePGPKey,
  servePGPKeyList,
} from "./serve_pgp.ts";

import { assertEquals } from "https://deno.land/std@0.204.0/assert/mod.ts";
import { ServerDependencies } from "./server.ts";

const emptyDependencies: ServerDependencies = {
  filterIncludesKey: () => false,
  parseParameters: () => ({}),
  getPGPTarget: () => undefined,
  servePGPKey: () => new Response(""),
  servePGPKeyList: () => new Response(""),
  sshKeys: [],
  pgpKeys: [],
};

Deno.test("servePGPKeyList: must return the list of key names", async () => {
  const pgpKeys = [
    { name: "key1", key: "key1" },
    { name: "key2", key: "key2" },
  ];
  const response = servePGPKeyList("1", { ...emptyDependencies, pgpKeys });
  assertEquals(response.status, 200);
  assertEquals(response.statusText, "OK");
  assertEquals(response.headers.get("X-Keys-Version"), "1");
  assertEquals(await response.text(), "key1\nkey2");
});

Deno.test("servePGPKeyList: must return an empty list if no keys are loaded", async () => {
  const response = servePGPKeyList("1", emptyDependencies);
  assertEquals(response.status, 200);
  assertEquals(response.statusText, "OK");
  assertEquals(response.headers.get("X-Keys-Version"), "1");
  assertEquals(await response.text(), "");
});

Deno.test("isValidPGPExtension: must return true for valid extensions", () => {
  assertEquals(isValidPGPExtension("asc"), true);
  assertEquals(isValidPGPExtension("pgp"), true);
  assertEquals(isValidPGPExtension("pub"), true);
});

Deno.test("isValidPGPExtension: must return false for invalid extensions", () => {
  assertEquals(isValidPGPExtension("invalid"), false);
  assertEquals(isValidPGPExtension("asc.invalid"), false);
  assertEquals(isValidPGPExtension("pgp.invalid"), false);
  assertEquals(isValidPGPExtension("pub.invalid"), false);
});

Deno.test("getPGPTarget: must return undefined for unknown routes", () => {
  assertEquals(getPGPTarget("/not_found"), undefined);
});

Deno.test("getPGPTarget: must return the key name and extension", () => {
  assertEquals(getPGPTarget("/pgp/key.pgp"), {
    name: "key",
    extension: "pgp",
  });
  assertEquals(getPGPTarget("/pgp/key.asc"), {
    name: "key",
    extension: "asc",
  });
});

Deno.test(
  "getPGPTarget: must return undefined for extension when not included",
  () => {
    assertEquals(getPGPTarget("/pgp/key"), {
      name: "key",
      extension: undefined,
    });
  },
);

Deno.test("getPGPTarget: must return undefined for invalid extensions", () => {
  assertEquals(getPGPTarget("/pgp/key.invalid"), undefined);
  assertEquals(getPGPTarget("/pgp/key.pgp.invalid"), undefined);
  assertEquals(getPGPTarget("/pgp/key.asc.invalid"), undefined);
});

Deno.test("servePGPKey: must return 404 for unknown keys", async () => {
  const response = await servePGPKey(
    { name: "unknown" },
    "1",
    { ...emptyDependencies, pgpKeys: [{ name: "non-matching", key: "key" }] },
  );
  assertEquals(response.status, 404);
  assertEquals(response.statusText, "Not Found");
});

Deno.test("servePGPKey: must return the key for known keys", async () => {
  const response = await servePGPKey(
    { name: "key" },
    "1",
    { ...emptyDependencies, pgpKeys: [{ name: "key", key: "key" }] },
  );
  assertEquals(response.status, 200);
  assertEquals(response.statusText, "OK");
  assertEquals(await response.text(), "key");
  assertEquals(response.headers.get("X-Keys-Version"), "1");
  assertEquals(response.headers.get("Content-Disposition"), null);
});

Deno.test("servePGPKey: must return the key with content disposition if an extension is provided", async () => {
  const response = await servePGPKey(
    { name: "key", extension: "asc" },
    "1",
    { ...emptyDependencies, pgpKeys: [{ name: "key", key: "key" }] },
  );
  assertEquals(response.status, 200);
  assertEquals(response.statusText, "OK");
  assertEquals(await response.text(), "key");
  assertEquals(response.headers.get("X-Keys-Version"), "1");
  assertEquals(
    response.headers.get("Content-Disposition"),
    'attachment; filename="key.asc"',
  );
});
