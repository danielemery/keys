import { accepts } from "@std/http/negotiation";

const supportedContentTypes = [
  "text/plain",
  "application/json",
  "text/html",
] as const;

export type ContentType = typeof supportedContentTypes[number];

export function isValidContentType(
  contentType: string | null,
): contentType is ContentType {
  return supportedContentTypes.includes(contentType as ContentType);
}

export function getContentType(
  acceptHeaderValue: Headers,
): ContentType {
  const contentType = accepts(
    { headers: acceptHeaderValue },
    ...supportedContentTypes,
  );

  if (contentType && isValidContentType(contentType)) {
    return contentType;
  }

  return "text/plain";
}
