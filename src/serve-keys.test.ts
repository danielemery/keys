import { assertEquals } from "https://deno.land/std@0.204.0/assert/mod.ts";
import {
  assertSpyCall,
  assertSpyCalls,
  spy,
} from "https://deno.land/std@0.204.0/testing/mock.ts";
import { ServerDependencies } from "./server.ts";
import { serveKeys } from "./serve-keys.ts";
import { PublicSSHKey } from "./load_config.ts";

const TEST_URL = "http://localhost";

const emptyDependencies: ServerDependencies = {
  filterIncludesKey: () => false,
  parseParameters: () => ({}),
  serveKeys: () => new Response(""),
  getPGPTarget: () => undefined,
  servePGPKey: () => new Response(""),
  servePGPKeyList: () => new Response(""),
  sshKeys: [],
  pgpKeys: [],
};

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
    });

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
