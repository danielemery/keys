import { assertEquals } from "https://deno.land/std@0.117.0/testing/asserts.ts";
import { Filter, filterIncludesKey, parseParameters } from "./filter.ts";

/** parseParameters */
Deno.test("parseParameters: must parse oneOf url param", () => {
  const expected: Filter = {
    oneOf: ["simple"],
  };
  const actual = parseParameters(new URL("http://domain.com/api?oneOf=simple"));
  assertEquals(actual, expected);
});

Deno.test("parseParameters: must parse complex filter params", () => {
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

/** filterIncludesKey */
Deno.test("filterIncludesKey: empty must filter correctly", () => {
  const filter: Filter = {};

  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["one-tag"],
      user: "user-one",
    }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["one-tag", "two-tags"],
      user: "user-one",
    }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: [], user: "user-one" }),
    true
  );
});

Deno.test("filterIncludesKey: oneOf must filter correctly", () => {
  const filter: Filter = {
    oneOf: ["one", "two"],
  };

  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: [], user: "user-one" }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["one"], user: "user-one" }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["two"], user: "user-one" }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["one", "two"],
      user: "user-one",
    }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["three"],
      user: "user-one",
    }),
    false
  );
});
Deno.test("filterIncludesKey: allOf must filter correctly", () => {
  const filter: Filter = {
    allOf: ["one", "two"],
  };

  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: [], user: "user-one" }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["one"], user: "user-one" }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["two"], user: "user-one" }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["one", "two"],
      user: "user-one",
    }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["one", "two", "three"],
      user: "user-one",
    }),
    true
  );
});
Deno.test("filterIncludesKey: noneOf must filter correctly", () => {
  const filter: Filter = {
    noneOf: ["one", "two"],
  };

  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: [], user: "user-one" }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["one"], user: "user-one" }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["two"], user: "user-one" }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["one", "two"],
      user: "user-one",
    }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["three"],
      user: "user-one",
    }),
    true
  );
});
Deno.test("filterIncludesKey: A mixture must filter correctly", () => {
  const filter: Filter = {
    noneOf: ["one", "two"],
    allOf: ["three"],
    oneOf: ["four", "five"],
  };

  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: [], user: "user-one" }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, { ...testKey, tags: ["one"], user: "user-one" }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["one", "two", "three"],
      user: "user-one",
    }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["three"],
      user: "user-one",
    }),
    false
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["three", "four"],
      user: "user-one",
    }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["three", "five"],
      user: "user-one",
    }),
    true
  );
  assertEquals(
    filterIncludesKey(filter, {
      ...testKey,
      tags: ["four", "five"],
      user: "user-one",
    }),
    false
  );
});
Deno.test("filterIncludesKey: User filter must filter correctly", () => {
  assertEquals(
    filterIncludesKey(
      { user: "user-one" },
      { ...testKey, tags: [], user: "user-one" }
    ),
    true
  );
  assertEquals(
    filterIncludesKey(
      { user: "user-two" },
      { ...testKey, tags: [], user: "user-one" }
    ),
    false
  );
  assertEquals(
    filterIncludesKey(
      { user: "user-one", allOf: ["match"] },
      { ...testKey, tags: ["match"], user: "user-one" }
    ),
    true
  );
  assertEquals(
    filterIncludesKey(
      { user: "user-two", allOf: ["match"] },
      { ...testKey, tags: ["match"], user: "user-one" }
    ),
    false
  );
  assertEquals(
    filterIncludesKey(
      { user: "user-one", allOf: ["no_match"] },
      { ...testKey, tags: ["match"], user: "user-one" }
    ),
    false
  );
  assertEquals(
    filterIncludesKey(
      { user: "user-two", allOf: ["no_match"] },
      { ...testKey, tags: ["match"], user: "user-one" }
    ),
    false
  );
});
