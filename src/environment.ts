import { z } from "../deps.ts";

const environmentSchema = z.object({
  PORT: z.string().regex(/^\d+$/).transform(Number).default("8000"),
  DOPPLER_ENVIRONMENT: z.string(),
  SENTRY_DSN: z.string().optional(),
});

export function parseEnvironmentVariables(variableObject: unknown) {
  return environmentSchema.parse(variableObject);
}
