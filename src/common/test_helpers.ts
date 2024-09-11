import { ServerDependencies } from "../shared_types/dependencies.ts";

/**
 * An empty set of dependencies.
 * Intended for use as a base in tests.
 */
export const emptyDependencies: ServerDependencies = {
  filterIncludesKey: () => false,
  parseParameters: () => ({}),
  serveHome: () => new Response(""),
  serveKeys: () => new Response(""),
  getPGPTarget: () => undefined,
  servePGPKey: () => new Response(""),
  servePGPKeyList: () => new Response(""),
  serveKnownHosts: () => new Response(""),
  sshKeys: [],
  pgpKeys: [],
  knownHosts: [],
  instanceName: "unit-tests",
};
