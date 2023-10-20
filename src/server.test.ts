import { handleRequest, ServerDependencies } from "./server.ts";

import {
  assertEquals,
  assertStringIncludes,
} from "https://deno.land/std@0.204.0/testing/asserts.ts";

const TEST_URL = "http://localhost";

const emptyDependencies: ServerDependencies = {
  filterIncludesKey: () => false,
  parseParameters: () => ({}),
  keys: [],
};

Deno.test(
  "handleRequest: must return 404 for unknown routes requests",
  async () => {
    const response = await handleRequest(
      new Request(`${TEST_URL}/not_found`),
      emptyDependencies,
    );

    assertEquals(response.status, 404);
    assertEquals(response.statusText, "Not Found");
  },
);

Deno.test("handleRequest: must return pgp key for /pgp.asc", async () => {
  const response = await handleRequest(
    new Request(`${TEST_URL}/pgp.asc`),
    emptyDependencies,
  );

  assertEquals(response.status, 200);
  assertEquals(response.statusText, "OK");
  assertStringIncludes(
    await response.text(),
    "-----BEGIN PGP PUBLIC KEY BLOCK-----",
  );
});
