import { parse } from "jsr:@std/yaml";
import { z } from "../../deps.ts";
import { loadFileContents } from "../common/file.ts";

export default async function loadConfig(path: string) {
  const contents = await loadFileContents(path);
  try {
    const config = parse(contents);
    const result = configSchema.parse(config);
    console.log(
      `Successfully loaded ${result["ssh-keys"].length} ssh public keys and ${
        result["pgp-keys"].length
      } pgp keys from ${path}.`,
    );
    return result;
  } catch (err) {
    console.error(`Failed to parse config file at path: ${path}`);
    throw err;
  }
}

const configSchema = z.object({
  "ssh-keys": z.array(
    z.object({
      name: z.string(),
      key: z.string(),
      user: z.string(),
      tags: z.array(z.string()).optional().default([]),
    }),
  ),
  "pgp-keys": z.array(z.object({ name: z.string(), key: z.string() }))
    .optional().default([]),
  "known-hosts": z.array(z.object({
    name: z.string().optional(),
    hosts: z.array(z.string()),
    keys: z.array(z.object({
      type: z.string(),
      key: z.string(),
      comment: z.string().optional(),
      revoked: z.boolean().optional().default(false),
      "cert-authority": z.boolean().optional().default(false),
    })),
  })).optional().default([]),
});

export type Config = z.infer<typeof configSchema>;
export type PublicSSHKey = z.infer<typeof configSchema>["ssh-keys"][number];
export type PGPKey = z.infer<typeof configSchema>["pgp-keys"][number];
export type KnownHost = z.infer<typeof configSchema>["known-hosts"][number];
