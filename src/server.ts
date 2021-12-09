import { serve } from '../deps.ts';

export default function start(port: number) {
  console.log(`http://localhost:${port}/`);
  serve((req) => new Response("Hello World\n"), { addr: `:${port}` });
}
