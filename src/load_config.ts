import { parse } from "jsr:@std/yaml";
import { z } from "../deps.ts";

export default async function loadConfig(path: string) {
  const file = await Deno.readFileSync(path);
  const text = new TextDecoder().decode(file);
  const config = parse(text);
  return configSchema.parse(config);
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
