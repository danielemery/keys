import { z } from "../../deps.ts";

const environmentSchema = z.object({
  PORT: z.string().regex(/^\d+$/).transform(Number).default("8000"),
  DOPPLER_ENVIRONMENT: z.string(),
  SENTRY_DSN: z.string().optional(),
  KEYS_VERSION: z.string(),
  CONFIG_PATH: z.string().optional().default("/config.yaml"),
  INSTANCE_NAME: z.string().optional().default("Unnamed"),
});

export function parseEnvironmentVariables(variableObject: unknown) {
  return environmentSchema.parse(variableObject);
}
