import { filterIncludesKey, parseParameters } from "./filter.ts";
import { PublicSSHKey } from "./load_config.ts";
import { PGPKey } from "./load_config.ts";
import { getPGPTarget, servePGPKey, servePGPKeyList } from "./serve_pgp.ts";
import { serveKeys } from "./serve-keys.ts";
import { serveHome } from "./serve-home.ts";

/**
 * The dependencies required by the server.
 * We use this to make it easier to mock the dependencies in tests.
 */
export interface ServerDependencies {
  instanceName: string;
  filterIncludesKey: typeof filterIncludesKey;
  parseParameters: typeof parseParameters;
  serveHome: typeof serveHome;
  serveKeys: typeof serveKeys;
  getPGPTarget: typeof getPGPTarget;
  servePGPKey: typeof servePGPKey;
  servePGPKeyList: typeof servePGPKeyList;
  sshKeys: PublicSSHKey[];
  pgpKeys: PGPKey[];
}
