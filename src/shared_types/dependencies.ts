import { filterIncludesKey, parseParameters } from "../routes/keys/filter.ts";
import { KnownHost, PGPKey, PublicSSHKey } from "../config/load_config.ts";
import {
  getPGPTarget,
  servePGPKey,
  servePGPKeyList,
} from "../routes/pgp/serve_pgp.ts";
import { serveKeys } from "../routes/keys/serve-keys.ts";
import { serveHome } from "../routes/serve-home.ts";

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
  knownHosts: KnownHost[];
}
