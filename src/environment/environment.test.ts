import { assertEquals, assertThrows } from "@std/assert";
import { parseEnvironmentVariables } from "../environment/environment.ts";
import { ZodError } from "../../deps.ts";

const baseVariables = {
  DOPPLER_ENVIRONMENT: "unit_tests",
  KEYS_VERSION: "unit_tests",
  CONFIG_PATH: "/test.yaml",
  INSTANCE_NAME: "Test",
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
  "parseEnvironmentVariables: must use defaults if optional variables are not supplied",
  () => {
    const variables = {
      DOPPLER_ENVIRONMENT: "unit_tests",
      KEYS_VERSION: "unit_tests",
    };

    assertEquals(parseEnvironmentVariables(variables), {
      CONFIG_PATH: "/config.yaml",
      DOPPLER_ENVIRONMENT: "unit_tests",
      INSTANCE_NAME: "Unnamed",
      KEYS_VERSION: "unit_tests",
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
