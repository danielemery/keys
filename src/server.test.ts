import { handleRequest, ServerDependencies } from "./server.ts";
import { filterIncludesKey, parseParameters } from "./filter.ts";

import {
  assertEquals,
  assertStringIncludes,
} from "https://deno.land/std@0.204.0/testing/asserts.ts";
import {
  assertSpyCall,
  assertSpyCalls,
  spy,
} from "https://deno.land/std@0.204.0/testing/mock.ts";

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

Deno.test(
  "handleRequest: must call appropriate functions and return keys",
  async () => {
    const parseParametersStub = spy(parseParameters);
    const filterIncludesKeyStub = spy(filterIncludesKey);
    const fakeKeys = [
      {
        name: "key-1",
        user: "user",
        tags: ["private"],
        key: "ssh-rsa fake1",
      },
      {
        name: "key-2",
        user: "user",
        tags: ["public"],
        key: "ssh-rsa fake2",
      },
    ];

    const url = `${TEST_URL}/api?oneOf=private&noneOf=public&noneOf=github`;

    const response = await handleRequest(new Request(url), {
      parseParameters: parseParametersStub,
      filterIncludesKey: filterIncludesKeyStub,
      keys: fakeKeys,
    });

    assertSpyCalls(parseParametersStub, 1);
    assertSpyCall(parseParametersStub, 0, {
      args: [new URL(url)],
    });

    assertSpyCalls(filterIncludesKeyStub, 2);
    assertSpyCall(filterIncludesKeyStub, 0, {
      args: [{ oneOf: ["private"], noneOf: ["public", "github"] }, fakeKeys[0]],
    });

    assertEquals(response.status, 200);
    assertEquals(response.statusText, "OK");
    assertEquals(await response.text(), "ssh-rsa fake1 user@key-1");
  },
);
