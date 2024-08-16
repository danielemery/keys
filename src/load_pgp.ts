import { listDirectoryContents, loadFileContents } from "./file.ts";

export interface PGPKey {
  name: string;
  key: string;
}

export const validPGPExtensions = ["asc", "pub", "pgp"] as const;
export type ValidPGPExtension = typeof validPGPExtensions[number];
export const validPGPExtensionsString = `${
  validPGPExtensions.slice(0, -1).join(", ")
} and ${validPGPExtensions.slice(-1)[0]}`;

export default async function loadPGPKeys(directory: string) {
  const list = await listDirectoryContents(directory);
  const results: PGPKey[] = [];
  for await (const file of list) {
    if (file.isFile) {
      if (!file.name.endsWith(".pub") && !file.name.endsWith(".asc")) {
        console.warn(
          `Ignoring file ${file.name} in PGP directory, only ${validPGPExtensionsString} files are considered.`,
        );
      } else {
        const key = await loadFileContents(`${directory}/${file.name}`);
        results.push({ name: removeExtension(file.name), key });
      }
    } else {
      console.warn(
        `Ignoring directory ${file.name} in PGP directory, only files are considered.`,
      );
    }
  }
  console.log(
    `Successfully loaded ${results.length} public PGP keys from ${directory}.`,
  );
  return results;
}

function removeExtension(fileName: string) {
  return fileName.replace(/\.[^/.]+$/, "");
}
