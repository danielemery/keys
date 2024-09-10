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
