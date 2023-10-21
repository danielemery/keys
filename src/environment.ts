import { z } from "../deps.ts";

const environmentSchema = z.object({
  PORT: z.string().regex(/^\d+$/).transform(Number).default("8000"),
});

export function parseEnvironmentVariables(variableObject: unknown) {
  return environmentSchema.parse(variableObject);
}
