import { STATUS_CODE, STATUS_TEXT } from "@std/http";
import { ServerDependencies } from "../../shared_types/dependencies.ts";
import { ContentType } from "../../common/content-types.ts";

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
          "Content-Type": "text/plain",
        },
      });
    }
    case "application/json": {
      const jsonData = {
        version,
        keys: filteredKeys.map((key) => ({
          key: key.key,
          user: key.user,
          name: key.name,
          tags: key.tags,
        })),
      };

      return new Response(JSON.stringify(jsonData), {
        status: STATUS_CODE.OK,
        statusText: STATUS_TEXT[STATUS_CODE.OK],
        headers: {
          "X-Keys-Version": version,
          "Content-Type": "application/json",
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
