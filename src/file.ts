export async function loadFileContents(path: string) {
  try {
    const file = await Deno.readFileSync(path);
    const text = new TextDecoder().decode(file);
    return text;
  } catch (err) {
    console.error(`Failed to read file at path: ${path}`);
    throw err;
  }
}

export async function listDirectoryContents(path: string) {
  try {
    const files = await Deno.readDir(path);
    const result: Deno.DirEntry[] = [];
    for await (const file of files) {
      result.push(file);
    }
    return result;
  } catch (err) {
    console.warn(
      `Failed to list PGP directory contents at ${path}, PGP key serving will be disabled.`,
    );
    console.warn(err);
    return [];
  }
}
