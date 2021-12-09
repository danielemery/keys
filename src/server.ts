import { serve, Status, STATUS_TEXT } from "../deps.ts";
import { filterIncludesKey, parseParameters } from "./filter.ts";
import keys from "./public_keys.ts";

export default function start(port: number) {
  console.log(`Server listening at on port ${port}`);
  serve(
    (req: Request) => {
      const url = new URL(req.url);
      if (url.pathname !== "/api") {
        return new Response(undefined, {
          status: Status.NotFound,
          statusText: STATUS_TEXT.get(Status.NotFound),
        });
      }
      const filter = parseParameters(url);
      const filteredKeys = keys.filter((key) => filterIncludesKey(filter, key));
      const responseData = filteredKeys
        .map((key) => `${key.key} ${key.name}`)
        .join("\n");
      return new Response(responseData, {
        status: Status.OK,
        statusText: STATUS_TEXT.get(Status.OK),
      });
    },
    { addr: `:${port}` }
  );
}
