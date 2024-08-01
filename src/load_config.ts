import { parse } from "jsr:@std/yaml";
import { z } from "../deps.ts";

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

async function loadFileContents(path: string) {
  try {
    const file = await Deno.readFileSync(path);
    const text = new TextDecoder().decode(file);
    return text;
  } catch (err) {
    console.error(`Failed to read file at path: ${path}`);
    throw err;
  }
}

const configSchema = z.object({
  "ssh-keys": z.array(z.object({
    name: z.string(),
    key: z.string(),
    user: z.string(),
    tags: z.array(z.string()).optional().default([]),
  })),
});

export type Config = z.infer<typeof configSchema>;
export type PublicSSHKey = z.infer<typeof configSchema>["ssh-keys"][number];
