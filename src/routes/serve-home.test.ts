import { assertEquals } from "https://deno.land/std@0.204.0/assert/mod.ts";
import { serveHome } from "./serve-home.ts";
import { emptyDependencies } from "../common/test_helpers.ts";

Deno.test("serveKeys: must return 200 for valid requests", async () => {
  const response = await serveHome(
    "unit-tests",
    {
      ...emptyDependencies,
      sshKeys: [{
        key: "ssh-rsa fake1",
        name: "key-1",
        tags: ["private"],
        user: "demery",
      }],
    },
    "text/plain",
  );
  assertEquals(response.status, 200);
  assertEquals(response.statusText, "OK");
  const responseText = await response.text();
  assertEquals(
    responseText,
    `Welcome to the "unit-tests" keys instance.
There are 1 SSH keys available at /keys.
There are 0 PGP keys available at /pgp.
There are 0 known hosts available at /known_hosts.
This server is running version unit-tests.`,
  );
});

Deno.test("serveKeys: must return NotAcceptable for unsupported content type", async () => {
  const response = await serveHome(
    "unit-tests",
    emptyDependencies,
    "text/html",
  );
  assertEquals(response.status, 406);
  assertEquals(response.statusText, "Not Acceptable");
});
