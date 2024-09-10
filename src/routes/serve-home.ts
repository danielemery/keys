import { STATUS_CODE, STATUS_TEXT } from "@std/http";
import { ContentType } from "../common/content-types.ts";
import { ServerDependencies } from "../shared_types/dependencies.ts";

export function serveHome(
  version: string,
  dependencies: ServerDependencies,
  contentType: ContentType,
) {
  switch (contentType) {
    case "text/plain":
      return new Response(
        generateTextBody(version, dependencies),
        {
          status: 200,
          statusText: "OK",
          headers: {
            "X-Keys-Version": version,
          },
        },
      );
    default:
      return new Response(undefined, {
        status: STATUS_CODE.NotAcceptable,
        statusText: STATUS_TEXT[STATUS_CODE.NotAcceptable],
      });
  }
}

function generateTextBody(
  version: string,
  serverDependencies: ServerDependencies,
) {
  return `Welcome to the "${serverDependencies.instanceName}" keys instance.
There are ${serverDependencies.sshKeys.length} SSH keys available at /keys.
There are ${serverDependencies.pgpKeys.length} PGP keys available at /pgp.
This server is running version ${version}.`;
}
