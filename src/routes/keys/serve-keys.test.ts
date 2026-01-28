import { assertEquals } from "@std/assert";
import { assertSpyCall, assertSpyCalls, spy } from "@std/testing/mock";
import { serveKeys } from "./serve-keys.ts";
import { PublicSSHKey } from "../../config/load_config.ts";
import { emptyDependencies } from "../../common/test_helpers.ts";

const TEST_URL = "http://localhost";

const fakeKeys: PublicSSHKey[] = [
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

Deno.test(
  "serveKeys: must call appropriate functions and return keys for ssh key routes",
  async () => {
    const parseParametersSpy = spy(() => ({
      oneOf: ["private"],
      noneOf: ["public", "github"],
    }));
    const filterIncludesKeySpy = spy((_filter, key: PublicSSHKey) =>
      key.name === "key-1"
    );

    const url = `${TEST_URL}/keys?oneOf=private&noneOf=public&noneOf=github`;

    const response = await serveKeys(new URL(url), "unit-tests", {
      ...emptyDependencies,
      parseParameters: parseParametersSpy,
      filterIncludesKey: filterIncludesKeySpy,
      sshKeys: fakeKeys,
    }, "text/plain");

    assertSpyCalls(parseParametersSpy, 1);
    assertSpyCall(parseParametersSpy, 0, {
      args: [new URL(url)],
    });

    assertSpyCalls(filterIncludesKeySpy, 2);
    assertSpyCall(filterIncludesKeySpy, 0, {
      args: [
        { oneOf: ["private"], noneOf: ["public", "github"] },
        fakeKeys[0],
      ],
    });

    assertEquals(response.status, 200);
    assertEquals(response.statusText, "OK");
    assertEquals(await response.text(), "ssh-rsa fake1 user@key-1");
  },
);

Deno.test("serveKeys: must return NotAcceptable for unsupported content type", async () => {
  const response = await serveKeys(
    new URL(TEST_URL),
    "unit-tests",
    emptyDependencies,
    "text/html",
  );
  assertEquals(response.status, 406);
  assertEquals(response.statusText, "Not Acceptable");
});

Deno.test(
  "serveKeys (json): must return keys in JSON format",
  async () => {
    const parseParametersSpy = spy(() => ({
      oneOf: ["private"],
      noneOf: ["public", "github"],
    }));
    const filterIncludesKeySpy = spy((_filter, key: PublicSSHKey) =>
      key.name === "key-1"
    );

    const url = `${TEST_URL}/keys?oneOf=private&noneOf=public&noneOf=github`;

    const response = await serveKeys(new URL(url), "unit-tests", {
      ...emptyDependencies,
      parseParameters: parseParametersSpy,
      filterIncludesKey: filterIncludesKeySpy,
      sshKeys: fakeKeys,
    }, "application/json");

    assertSpyCalls(parseParametersSpy, 1);
    assertSpyCall(parseParametersSpy, 0, {
      args: [new URL(url)],
    });

    assertSpyCalls(filterIncludesKeySpy, 2);
    assertSpyCall(filterIncludesKeySpy, 0, {
      args: [
        { oneOf: ["private"], noneOf: ["public", "github"] },
        fakeKeys[0],
      ],
    });

    assertEquals(response.status, 200);
    assertEquals(response.statusText, "OK");
    assertEquals(response.headers.get("Content-Type"), "application/json");

    const jsonResponse = await response.json();
    assertEquals(jsonResponse.version, "unit-tests");
    assertEquals(jsonResponse.keys.length, 1);
    assertEquals(jsonResponse.keys[0].name, "key-1");
    assertEquals(jsonResponse.keys[0].user, "user");
    assertEquals(jsonResponse.keys[0].key, "ssh-rsa fake1");
    assertEquals(jsonResponse.keys[0].tags, ["private"]);
  },
);
