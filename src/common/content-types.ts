import { Accepts } from "../../deps.ts";

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
  const accept = new Accepts(acceptHeaderValue);

  const contentType = accept.types(supportedContentTypes.map((type) => type));

  if (Array.isArray(contentType)) {
    console.warn(
      'Multiple content types returned, this should not be possible, defaulting to "text/plain"',
    );
    return "text/plain";
  }

  if ((typeof contentType === "string") && isValidContentType(contentType)) {
    return contentType;
  }

  return "text/plain";
}
