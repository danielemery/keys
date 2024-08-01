import {
  assertEquals,
  assertThrows,
} from "https://deno.land/std@0.117.0/testing/asserts.ts";
import { parseEnvironmentVariables } from "./environment.ts";
import { ZodError } from "../deps.ts";

const baseVariables = {
  DOPPLER_ENVIRONMENT: "unit_tests",
  KEYS_VERSION: "unit_tests",
  CONFIG_PATH: "/test.yaml",
};

Deno.test(
  "parseEnvironmentVariables: must convert string variables to correct types",
  () => {
    const variables = {
      ...baseVariables,
      PORT: "1234",
    };

    assertEquals(parseEnvironmentVariables(variables), {
      ...baseVariables,
      PORT: 1234,
    });
  },
);
Deno.test(
  "parseEnvironmentVariables: must use defaults if variables are not supplied",
  () => {
    const variables = {
      ...baseVariables,
    };

    assertEquals(parseEnvironmentVariables(variables), {
      ...baseVariables,
      PORT: 8000,
    });
  },
);
Deno.test(
  "parseEnvironmentVariables: must throw ZodError if input is invalid",
  () => {
    const variables = {
      ...baseVariables,
      PORT: "not-a-number",
    };

    assertThrows(() => parseEnvironmentVariables(variables), ZodError);
  },
);
