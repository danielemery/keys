import { STATUS_CODE, STATUS_TEXT } from "@std/http";
import { ServerDependencies } from "./server.ts";
import {
  ValidPGPExtension,
  validPGPExtensions,
  validPGPExtensionsString,
} from "./load_pgp.ts";
import { ContentType } from "./content-types.ts";

export function servePGPKeyList(
  version: string,
  dependencies: ServerDependencies,
  contentType: ContentType,
) {
  switch (contentType) {
    case "text/plain": {
      const resultString = dependencies.pgpKeys.map((key) => key.name).join(
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

interface PGPKeyTarget {
  name: string;
  extension?: ValidPGPExtension;
}

export function isValidPGPExtension(
  extension: string,
): extension is ValidPGPExtension {
  return validPGPExtensions.includes(extension as ValidPGPExtension);
}

/**
 * Get the pgp key target from the url pathname.
 * @param pathname The url pathname to use to determine the pgp key target and optionally the file extension.
 * @returns The name of the pgp key target and optionally the file extension. (or undefined if the pathname does not start with /pgp)
 */
export function getPGPTarget(pathname: string): PGPKeyTarget | undefined {
  const match = pathname.match(/^\/pgp\/([^/.]+)(?:\.([^/]+))?$/);
  if (!match) {
    return;
  }
  if (match[2] !== undefined && !isValidPGPExtension(match[2])) {
    console.warn(
      `Ignoring key ${pathname}, only ${validPGPExtensionsString} files are considered.`,
    );
    return;
  }
  return { name: match[1], extension: match[2] };
}

export function servePGPKey(
  target: PGPKeyTarget,
  version: string,
  dependencies: ServerDependencies,
  contentType: ContentType,
) {
  const key = dependencies.pgpKeys.find((key) => key.name === target.name);
  if (!key) {
    console.log(`Key not found: ${target}`);
    return new Response(undefined, {
      status: STATUS_CODE.NotFound,
      statusText: STATUS_TEXT[STATUS_CODE.NotFound],
    });
  }

  switch (contentType) {
    case "text/plain":
      return new Response(key.key, {
        status: STATUS_CODE.OK,
        statusText: STATUS_TEXT[STATUS_CODE.OK],
        headers: {
          "X-Keys-Version": version,
          ...(target.extension
            ? {
              "Content-Disposition":
                `attachment; filename="${key.name}.${target.extension}"`,
            }
            : {}),
        },
      });
    default:
      return new Response(undefined, {
        status: STATUS_CODE.NotAcceptable,
        statusText: STATUS_TEXT[STATUS_CODE.NotAcceptable],
      });
  }
}
