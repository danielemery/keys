import { handleRequest, ServerDependencies } from "./server.ts";
import { filterIncludesKey, parseParameters } from "./filter.ts";
import { servePGPKeyList } from "./serve_pgp.ts";

import { assertEquals } from "https://deno.land/std@0.204.0/assert/mod.ts";
import {
  assertSpyCall,
  assertSpyCalls,
  spy,
} from "https://deno.land/std@0.204.0/testing/mock.ts";

const TEST_URL = "http://localhost";

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

const emptyDependencies: ServerDependencies = {
  filterIncludesKey: () => false,
  parseParameters: () => ({}),
  getPGPTarget: () => undefined,
  servePGPKey: () => new Response(""),
  servePGPKeyList: () => new Response(""),
  sshKeys: [],
  pgpKeys: [],
};

Deno.test(
  "handleRequest: must return 404 for unknown routes requests",
  async () => {
    const response = await handleRequest(
      new Request(`${TEST_URL}/not_found`),
      emptyDependencies,
      "unit_tests",
    );

    assertEquals(response.status, 404);
    assertEquals(response.statusText, "Not Found");
  },
);

Deno.test(
  "handleRequest: must call appropriate functions and return keys for ssh key routes",
  async () => {
    const parseParametersSpy = spy(parseParameters);
    const filterIncludesKeySpy = spy(filterIncludesKey);

    const url = `${TEST_URL}/keys?oneOf=private&noneOf=public&noneOf=github`;

    const response = await handleRequest(new Request(url), {
      ...emptyDependencies,
      parseParameters: parseParametersSpy,
      filterIncludesKey: filterIncludesKeySpy,
      sshKeys: fakeKeys,
    }, "unit_tests");

    assertSpyCalls(parseParametersSpy, 1);
    assertSpyCall(parseParametersSpy, 0, {
      args: [new URL(url)],
    });

    assertSpyCalls(filterIncludesKeySpy, 2);
    assertSpyCall(filterIncludesKeySpy, 0, {
      args: [{ oneOf: ["private"], noneOf: ["public", "github"] }, fakeKeys[0]],
    });

    assertEquals(response.status, 200);
    assertEquals(response.statusText, "OK");
    assertEquals(await response.text(), "ssh-rsa fake1 user@key-1");
  },
);

Deno.test("handleRequest: must call appropriate functions and return keys for pgp key list routes", async () => {
  const servePGPKeyListSpy = spy(servePGPKeyList);

  const url = `${TEST_URL}/pgp`;

  const dependencies = {
    ...emptyDependencies,
    servePGPKeyList: servePGPKeyListSpy,
    pgpKeys: fakeKeys,
  };

  const response = await handleRequest(
    new Request(url),
    dependencies,
    "unit_tests",
  );

  assertSpyCalls(servePGPKeyListSpy, 1);
  assertSpyCall(servePGPKeyListSpy, 0, {
    args: ["unit_tests", dependencies],
  });

  assertEquals(response.status, 200);
  assertEquals(response.statusText, "OK");
  assertEquals(
    await response.text(),
    `key-1
key-2`,
  );
});

Deno.test("handleRequest: must call appropriate functions and return keys for pgp key routes", async () => {
  const getPGPTargetSpy = spy(() => ({
    name: "key-1",
    extension: "asc" as const,
  }));
  const servePGPKeySpy = spy(() => new Response("fake"));

  const url = `${TEST_URL}/pgp/key-1.asc`;

  const dependencies = {
    ...emptyDependencies,
    getPGPTarget: getPGPTargetSpy,
    servePGPKey: servePGPKeySpy,
    pgpKeys: fakeKeys,
  };

  const response = await handleRequest(
    new Request(url),
    dependencies,
    "unit_tests",
  );

  assertSpyCalls(getPGPTargetSpy, 1);
  assertSpyCall(getPGPTargetSpy, 0, {
    args: ["/pgp/key-1.asc"],
  });

  assertSpyCalls(servePGPKeySpy, 1);
  assertSpyCall(servePGPKeySpy, 0, {
    args: [{ name: "key-1", extension: "asc" }, "unit_tests", dependencies],
  });

  assertEquals(response.status, 200);
  assertEquals(await response.text(), "fake");
});

Deno.test(
  "handleRequest: must return 500 if unexpected error is thrown",
  async () => {
    const throwingParseParameters: typeof parseParameters = () => {
      throw new Error("Unexpected error");
    };
    const parseParametersStub = spy(throwingParseParameters);
    const filterIncludesKeyStub = spy(filterIncludesKey);

    const url = `${TEST_URL}/keys?oneOf=private&noneOf=public&noneOf=github`;

    const response = await handleRequest(new Request(url), {
      ...emptyDependencies,
      parseParameters: parseParametersStub,
      filterIncludesKey: filterIncludesKeyStub,
      sshKeys: fakeKeys,
    }, "unit_tests");

    assertSpyCalls(parseParametersStub, 1);
    assertSpyCall(parseParametersStub, 0, {
      args: [new URL(url)],
    });

    assertSpyCalls(filterIncludesKeyStub, 0);

    assertEquals(response.status, 500);
    assertEquals(response.statusText, "Internal Server Error");
  },
);
