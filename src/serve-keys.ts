import { STATUS_CODE, STATUS_TEXT } from "@std/http";
import { ServerDependencies } from "./server.ts";

export function serveKeys(
  url: URL,
  version: string,
  dependencies: ServerDependencies,
) {
  const { filterIncludesKey, parseParameters, sshKeys } = dependencies;

  const filter = parseParameters(url);
  const filteredKeys = sshKeys.filter((key) => filterIncludesKey(filter, key));

  const responseData = filteredKeys
    .map((key) => `${key.key} ${key.user}@${key.name}`)
    .join("\n");

  return new Response(responseData, {
    status: STATUS_CODE.OK,
    statusText: STATUS_TEXT[STATUS_CODE.OK],
    headers: {
      "X-Keys-Version": version,
    },
  });
}
