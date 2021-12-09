import { assertEquals } from "https://deno.land/std@0.117.0/testing/asserts.ts";
import { Filter, filterIncludesKey, parseParameters } from "./filter.ts";

Deno.test("must parse oneOf url param", () => {
  const expected: Filter = {
    oneOf: ["simple"],
  };
  const actual = parseParameters(new URL("http://domain.com/api?oneOf=simple"));
  assertEquals(actual, expected);
});

Deno.test("must parse complex filter params", () => {
  const expected: Filter = {
    noneOf: ["not-me", "or-me"],
    allOf: ["definitely-me", "and-me"],
  };
  const actual = parseParameters(
    new URL(
      "http://domain.com/api?noneOf=not-me&noneOf=or-me&allOf=definitely-me&allOf=and-me"
    )
  );
  assertEquals(actual, expected);
});

const testKey = {
  key: "",
  name: "",
};

Deno.test("empty must filter correctly", () => {
  const filter: Filter = {};

  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["one-tag"] }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["one-tag", "two-tags"] }),
    true
  );
  assertEquals(filterIncludesKey(filter, { ...testKey, tags: [] }), true);
});

Deno.test("oneOf must filter correctly", () => {
  const filter: Filter = {
    oneOf: ["one", "two"],
  };

  assertEquals(filterIncludesKey(filter, { ...testKey, tags: [] }), false);
  assertEquals(filterIncludesKey(filter, { ...testKey, tags: ["one"] }), true);
  assertEquals(filterIncludesKey(filter, { ...testKey, tags: ["two"] }), true);
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["one", "two"] }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["three"] }),
    false
  );
});
Deno.test("allOf must filter correctly", () => {
  const filter: Filter = {
    allOf: ["one", "two"],
  };

  assertEquals(filterIncludesKey(filter, { ...testKey, tags: [] }), false);
  assertEquals(filterIncludesKey(filter, { ...testKey, tags: ["one"] }), false);
  assertEquals(filterIncludesKey(filter, { ...testKey, tags: ["two"] }), false);
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["one", "two"] }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["one", "two", "three"] }),
    true
  );
});
Deno.test("noneOf must filter correctly", () => {
  const filter: Filter = {
    noneOf: ["one", "two"],
  };

  assertEquals(filterIncludesKey(filter, { ...testKey, tags: [] }), true);
  assertEquals(filterIncludesKey(filter, { ...testKey, tags: ["one"] }), false);
  assertEquals(filterIncludesKey(filter, { ...testKey, tags: ["two"] }), false);
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["one", "two"] }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["three"] }),
    true
  );
});
Deno.test("A mixture must filter correctly", () => {
  const filter: Filter = {
    noneOf: ["one", "two"],
    allOf: ["three"],
    oneOf: ["four", "five"],
  };

  assertEquals(filterIncludesKey(filter, { ...testKey, tags: [] }), false);
  assertEquals(filterIncludesKey(filter, { ...testKey, tags: ["one"] }), false);
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["one", "two", "three"] }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["three"] }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["three", "four"] }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["three", "five"] }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["four", "five"] }),
    false
  );
});
