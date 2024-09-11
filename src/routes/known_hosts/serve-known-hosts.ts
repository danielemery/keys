import { STATUS_CODE, STATUS_TEXT } from "@std/http";
import { ContentType } from "../../common/content-types.ts";
import { ServerDependencies } from "../../shared_types/dependencies.ts";
import { KnownHost } from "../../config/load_config.ts";

export function serveKnownHosts(
  version: string,
  dependencies: ServerDependencies,
  contentType: ContentType,
) {
  switch (contentType) {
    case "text/plain": {
      const resultString = dependencies.knownHosts.map((knownHost) =>
        knownHost.keys.map((key) => {
          return `${getMarker(key)}${
            knownHost.hosts.join(",")
          } ${key.type} ${key.key}${key.comment ? ` ${key.comment}` : ""}`;
        }).join("\n")
      ).flat().join(
        "\n",
      );
      return new Response(resultString, {
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

function getMarker(key: KnownHost["keys"][0]) {
  if (key["cert-authority"]) {
    return "@cert-authority ";
  }
  if (key.revoked) {
    return "@revoked ";
  }
  return "";
}
