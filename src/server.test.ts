import { handleRequest } from "./server.ts";
import { servePGPKeyList } from "./routes/pgp/serve_pgp.ts";
import { serveKeys } from "./routes/keys/serve-keys.ts";
import { ServerDependencies } from "./shared_types/dependencies.ts";
import { emptyDependencies } from "./common/test_helpers.ts";

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

const acceptPlainHeaders = new Headers({
  "Accept": "text/plain",
});

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

Deno.test('handleRequest: must call appropriate functions and return keys for "/" (home) route', async () => {
  const serveHomeSpy = spy(() => new Response("fake response"));

  const url = `${TEST_URL}/`;

  const dependencies: ServerDependencies = {
    ...emptyDependencies,
    serveHome: serveHomeSpy,
  };

  const response = await handleRequest(
    new Request(url, { headers: acceptPlainHeaders }),
    dependencies,
    "unit_tests",
  );

  assertSpyCalls(serveHomeSpy, 1);
  assertSpyCall(serveHomeSpy, 0, {
    args: ["unit_tests", dependencies, "text/plain"],
  });

  assertEquals(response.status, 200);
  assertEquals(await response.text(), "fake response");
});

Deno.test(
  "handleRequest: must call appropriate functions and return keys for ssh key routes",
  async () => {
    const serveKeysSpy = spy(() => new Response("fake response"));

    const url = `${TEST_URL}/keys?oneOf=private&noneOf=public&noneOf=github`;

    const dependencies: ServerDependencies = {
      ...emptyDependencies,
      serveKeys: serveKeysSpy,
      sshKeys: fakeKeys,
    };

    const response = await handleRequest(
      new Request(url, { headers: acceptPlainHeaders }),
      dependencies,
      "unit_tests",
    );

    assertSpyCalls(serveKeysSpy, 1);
    assertSpyCall(serveKeysSpy, 0, {
      args: [new URL(url), "unit_tests", dependencies, "text/plain"],
    });

    assertEquals(response.status, 200);
    assertEquals(await response.text(), "fake response");
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
    new Request(url, { headers: acceptPlainHeaders }),
    dependencies,
    "unit_tests",
  );

  assertSpyCalls(servePGPKeyListSpy, 1);
  assertSpyCall(servePGPKeyListSpy, 0, {
    args: ["unit_tests", dependencies, "text/plain"],
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
    new Request(url, { headers: acceptPlainHeaders }),
    dependencies,
    "unit_tests",
  );

  assertSpyCalls(getPGPTargetSpy, 1);
  assertSpyCall(getPGPTargetSpy, 0, {
    args: ["/pgp/key-1.asc"],
  });

  assertSpyCalls(servePGPKeySpy, 1);
  assertSpyCall(servePGPKeySpy, 0, {
    args: [
      { name: "key-1", extension: "asc" },
      "unit_tests",
      dependencies,
      "text/plain",
    ],
  });

  assertEquals(response.status, 200);
  assertEquals(await response.text(), "fake");
});

Deno.test('handleRequest: must call appropriate functions and return known hosts for "/known_hosts" route', async () => {
  const serveKnownHostsSpy = spy(() => new Response("fake response"));

  const url = `${TEST_URL}/known_hosts`;

  const dependencies: ServerDependencies = {
    ...emptyDependencies,
    serveKnownHosts: serveKnownHostsSpy,
    knownHosts: [],
  };

  const response = await handleRequest(
    new Request(url, { headers: acceptPlainHeaders }),
    dependencies,
    "unit_tests",
  );

  assertSpyCalls(serveKnownHostsSpy, 1);
  assertSpyCall(serveKnownHostsSpy, 0, {
    args: ["unit_tests", dependencies, "text/plain"],
  });

  assertEquals(response.status, 200);
  assertEquals(await response.text(), "fake response");
});

Deno.test(
  "handleRequest: must return 500 if unexpected error is thrown",
  async () => {
    const throwingServeKeys: typeof serveKeys = () => {
      throw new Error("Unexpected error");
    };
    const serveKeysStub = spy(throwingServeKeys);

    const url = `${TEST_URL}/keys?oneOf=private&noneOf=public&noneOf=github`;

    const dependencies = {
      ...emptyDependencies,
      serveKeys: serveKeysStub,
      sshKeys: fakeKeys,
    };

    const response = await handleRequest(
      new Request(url, { headers: acceptPlainHeaders }),
      dependencies,
      "unit_tests",
    );

    assertSpyCalls(serveKeysStub, 1);
    assertSpyCall(serveKeysStub, 0, {
      args: [new URL(url), "unit_tests", dependencies, "text/plain"],
    });

    assertEquals(response.status, 500);
    assertEquals(response.statusText, "Internal Server Error");
  },
);
