import { PublicKey } from "./public_keys.ts";

/** Available filter options. */
export interface Filter {
  /** If included only keys with ALL the tags provided will be returned. */
  allOf?: string[];
  /** If included only keys with AT LEAST ONE of the tags provided will be returned. */
  oneOf?: string[];
  /** If included only keys with NON of the tags provided will be returned. */
  noneOf?: string[];
}

/**
 * Get whether or not the given key is included in the given filter.
 * @param filter The filter to evaluate the key against. See Filter type for details.
 * @param key The key to check against the filter.
 * @returns true if the filter includes the key, false if not.
 */
export function filterIncludesKey(filter: Filter, key: PublicKey) {
  if (
    filter.allOf &&
    filter.allOf.find((needle) => !key.tags.find((tag) => tag === needle))
  ) {
    return false;
  }

  if (
    filter.oneOf &&
    !key.tags.find((needle) => filter.oneOf?.includes(needle))
  ) {
    return false;
  }

  if (
    filter.noneOf &&
    key.tags.find((needle) => filter.noneOf?.includes(needle))
  ) {
    return false;
  }

  return true;
}

/**
 * Parse the given url to determine the filter options.
 * @param url The url to parse.
 * @returns The filter defined by the url query params.
 */
export function parseParameters(url: URL): Filter {
  const params = new URLSearchParams(url.search);
  const filter: Filter = {};
  if (params.get("allOf") !== null) {
    filter.allOf = params.getAll("allOf");
  }
  if (params.get("oneOf") !== null) {
    filter.oneOf = params.getAll("oneOf");
  }
  if (params.get("noneOf") !== null) {
    filter.noneOf = params.getAll("noneOf");
  }
  return filter;
}
