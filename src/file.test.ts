import {
  assertEquals,
  assertRejects,
} from "https://deno.land/std@0.117.0/testing/asserts.ts";
import { loadFileContents } from "./file.ts";

Deno.test("loadFileContents: must return file contents", async () => {
  const text = await loadFileContents("./fixtures/missing-key.yaml");
  assertEquals(text, "some_key: some_value\n");
});

Deno.test("loadFileContents: must throw error for missing file", async () => {
  await assertRejects(
    async () => await loadFileContents("./fixtures/nothing.yaml"),
    Deno.errors.NotFound,
  );
});
