import {
  assertEquals,
  assertThrows,
} from "https://deno.land/std@0.117.0/testing/asserts.ts";
import { parseEnvironmentVariables } from "./environment.ts";
import { ZodError } from "../deps.ts";

Deno.test(
  "parseEnvironmentVariables: must convert string variables to correct types",
  () => {
    const variables = {
      PORT: "1234",
    };

    assertEquals(parseEnvironmentVariables(variables), { PORT: 1234 });
  }
);
Deno.test(
  "parseEnvironmentVariables: must use defaults if variables are not supplied",
  () => {
    const variables = {};

    assertEquals(parseEnvironmentVariables(variables), { PORT: 8000 });
  }
);
Deno.test("parseEnvironmentVariables: must throw ZodError if input is invalid", () => {
  const variables = {
    PORT: "not-a-number",
  };

  assertThrows(() => parseEnvironmentVariables(variables), ZodError);
});
