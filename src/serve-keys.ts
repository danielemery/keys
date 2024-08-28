import { STATUS_CODE, STATUS_TEXT } from "@std/http";
import { ServerDependencies } from "./dependencies.ts";
import { ContentType } from "./content-types.ts";

export function serveKeys(
  url: URL,
  version: string,
  dependencies: ServerDependencies,
  contentType: ContentType,
) {
  const { filterIncludesKey, parseParameters, sshKeys } = dependencies;

  const filter = parseParameters(url);
  const filteredKeys = sshKeys.filter((key) => filterIncludesKey(filter, key));

  switch (contentType) {
    case "text/plain": {
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
    default:
      return new Response(undefined, {
        status: STATUS_CODE.NotAcceptable,
        statusText: STATUS_TEXT[STATUS_CODE.NotAcceptable],
      });
  }
}
