import { assertEquals } from "https://deno.land/std@0.117.0/testing/asserts.ts";
import { getContentType, isValidContentType } from "./content-types.ts";

Deno.test("isValidContentType: should return true for valid content types", () => {
  const validContentTypes = [
    "text/html",
    "application/json",
    "text/plain",
  ];
  for (const contentType of validContentTypes) {
    const result = isValidContentType(contentType);
    assertEquals(result, true);
  }
});

Deno.test("isValidContentType: should return false for invalid content types", () => {
  const invalidContentTypes = [
    "text/invalid",
    "application/invalid",
    "invalid/plain",
  ];
  for (const contentType of invalidContentTypes) {
    const result = isValidContentType(contentType);
    assertEquals(result, false);
  }
});

Deno.test('getContentType: should return "text/plain" if no valid content type is found or if specifically requested', () => {
  const acceptInvalidHeaderValues = new Headers({
    "Accept": "text/invalid, application/invalid, invalid/plain",
  });
  const invalidResult = getContentType(acceptInvalidHeaderValues);
  assertEquals(invalidResult, "text/plain");

  const acceptPlainTextHeaderValue = new Headers({
    "Accept": "text/plain",
  });
  const plainTextResult = getContentType(acceptPlainTextHeaderValue);
  assertEquals(plainTextResult, "text/plain");
});

Deno.test('getContentType: should return "text/html" for a typical browser request', () => {
  const acceptHeaderValue = new Headers({
    "Accept":
      "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
  });
  const result = getContentType(acceptHeaderValue);
  assertEquals(result, "text/html");
});

Deno.test('getContentType: should return "application/json" for a JSON request', () => {
  const acceptHeaderValue = new Headers({
    "Accept": "application/json",
  });
  const result = getContentType(acceptHeaderValue);
  assertEquals(result, "application/json");
});

Deno.test('getContentType: should return "text/plain" for a wildcard (eg. curl) request', () => {
  const acceptHeaderValue = new Headers({
    "Accept": "*/*",
  });
  const result = getContentType(acceptHeaderValue);
  assertEquals(result, "text/plain");
});
