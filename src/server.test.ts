import { handleRequest } from "./server.ts";

import {
  assertEquals,
  assertStringIncludes,
} from "https://deno.land/std@0.117.0/testing/asserts.ts";

const TEST_URL = "http://localhost";

Deno.test(
  "handleRequest: should return 404 for unknown routes requests",
  async () => {
    const response = await handleRequest(new Request(`${TEST_URL}/not_found`));

    assertEquals(response.status, 404);
    assertEquals(response.statusText, "Not Found");
  }
);

Deno.test("handleRequest: should return pgp key for /pgp.asc", async () => {
  const response = await handleRequest(new Request(`${TEST_URL}/pgp.asc`));

  assertEquals(response.status, 200);
  assertEquals(response.statusText, "OK");
  assertStringIncludes(
    await response.text(),
    "-----BEGIN PGP PUBLIC KEY BLOCK-----"
  );
});
