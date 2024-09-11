import { assertEquals } from "https://deno.land/std@0.204.0/assert/mod.ts";
import { serveKnownHosts } from "./serve-known-hosts.ts";
import { emptyDependencies } from "../../common/test_helpers.ts";

Deno.test("serveKnownHosts (plain): must serve empty string if there are no dependencies", async () => {
  const actual = serveKnownHosts("unit-tests", {
    ...emptyDependencies,
  }, "text/plain");
  assertEquals(actual.status, 200);
  assertEquals(actual.statusText, "OK");
  assertEquals(await actual.text(), "");
});

Deno.test("serveKnownHosts (plain): must serve known hosts", async () => {
  const knownHosts = [
    {
      name: "host1",
      hosts: ["host1.com", "host2.com"],
      keys: [
        {
          type: "ssh-rsa",
          key: "key1",
          comment: "comment1",
          revoked: false,
          "cert-authority": false,
        },
        {
          type: "ssh-rsa",
          key: "key2",
          revoked: false,
          "cert-authority": false,
        },
      ],
    },
    {
      name: "host2",
      hosts: ["host3.com"],
      keys: [
        {
          type: "ssh-rsa",
          key: "key3",
          revoked: false,
          "cert-authority": false,
        },
      ],
    },
  ];
  const actual = serveKnownHosts("unit-tests", {
    ...emptyDependencies,
    knownHosts,
  }, "text/plain");
  assertEquals(actual.status, 200);
  assertEquals(actual.statusText, "OK");
  assertEquals(
    await actual.text(),
    `host1.com,host2.com ssh-rsa key1 comment1
host1.com,host2.com ssh-rsa key2
host3.com ssh-rsa key3`,
  );
});

Deno.test("serveKnownHosts (plain): must serve known hosts with markers", async () => {
  const knownHosts = [
    {
      name: "host1",
      hosts: ["host1.com"],
      keys: [
        {
          type: "ssh-rsa",
          key: "key1",
          revoked: false,
          "cert-authority": true,
        },
        {
          type: "ssh-rsa",
          key: "key2",
          revoked: true,
          "cert-authority": false,
          comment: "revoked 2024-09-11",
        },
      ],
    },
  ];
  const actual = serveKnownHosts("unit-tests", {
    ...emptyDependencies,
    knownHosts,
  }, "text/plain");
  assertEquals(actual.status, 200);
  assertEquals(actual.statusText, "OK");
  assertEquals(
    await actual.text(),
    `@cert-authority host1.com ssh-rsa key1
@revoked host1.com ssh-rsa key2 revoked 2024-09-11`,
  );
});

Deno.test("serveKnownHosts (plain): must return NotAcceptable for unsupported content type", () => {
  const actual = serveKnownHosts("unit-tests", emptyDependencies, "text/html");
  assertEquals(actual.status, 406);
  assertEquals(actual.statusText, "Not Acceptable");
});
