import { parse } from "jsr:@std/yaml";
import { z } from "../deps.ts";
import { loadFileContents } from "./file.ts";

export default async function loadConfig(path: string) {
  const contents = await loadFileContents(path);
  try {
    const config = parse(contents);
    return configSchema.parse(config);
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
});

export type Config = z.infer<typeof configSchema>;
export type PublicSSHKey = z.infer<typeof configSchema>["ssh-keys"][number];
