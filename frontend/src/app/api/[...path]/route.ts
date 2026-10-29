// Keep the browser on the website's origin; backend addressing is server configuration.
export const runtime = "nodejs";
export const dynamic = "force-dynamic";
export const maxDuration = 60;

type Context = { params: Promise<{ path: string[] }> };
const MAX_BODY = 256 * 1024;

async function proxy(request: Request, context: Context): Promise<Response> {
  const { path } = await context.params;
  const base = process.env.API_PROXY_URL || "http://127.0.0.1:8080";
  const destination = new URL(base);
  // Encode individual segments: incoming paths cannot replace the configured host.
  destination.pathname = `${destination.pathname.replace(/\/$/, "")}/${path.map(encodeURIComponent).join("/")}`;
  destination.search = new URL(request.url).search;
  const headers = new Headers();
  for (const name of [
    "authorization",
    "cookie",
    "content-type",
    "accept",
    "origin",
  ]) {
    const value = request.headers.get(name);
    if (value) headers.set(name, value);
  }
  let body: Uint8Array | undefined;
  if (request.body && !["GET", "HEAD"].includes(request.method)) {
    const reader = request.body.getReader();
    const chunks: Uint8Array[] = [];
    let length = 0;
    for (;;) {
      const chunk = await reader.read();
      if (chunk.done) break;
      length += chunk.value.byteLength;
      if (length > MAX_BODY) {
        await reader.cancel();
        return Response.json({ error: "Request too large" }, { status: 413 });
      }
      chunks.push(chunk.value);
    }
    body = new Uint8Array(length);
    let offset = 0;
    for (const chunk of chunks) {
      body.set(chunk, offset);
      offset += chunk.byteLength;
    }
  }
  try {
    const upstream = await fetch(destination, {
      method: request.method,
      headers,
      body: body as BodyInit | undefined,
      redirect: "manual",
      cache: "no-store",
      signal: AbortSignal.timeout(45000),
    });
    if (
      upstream.ok &&
      !(upstream.headers.get("content-type") || "").includes("application/json")
    ) {
      return Response.json(
        {
          error:
            "The testing server may be waking up. Check your inbox before retrying a write, and try again in about a minute.",
        },
        {
          status: 503,
          headers: { "Retry-After": "30", "Cache-Control": "no-store" },
        },
      );
    }
    const responseHeaders = new Headers({ "Cache-Control": "no-store" });
    const contentType = upstream.headers.get("content-type");
    if (contentType) responseHeaders.set("Content-Type", contentType);
    for (const cookie of upstream.headers.getSetCookie())
      responseHeaders.append("Set-Cookie", cookie);
    return new Response(upstream.body, {
      status: upstream.status,
      headers: responseHeaders,
    });
  } catch {
    console.error("Support API connection unavailable");
    return Response.json(
      {
        error:
          "The support service is temporarily unavailable. Please try again shortly.",
      },
      { status: 502 },
    );
  }
}

export {
  proxy as GET,
  proxy as POST,
  proxy as PATCH,
  proxy as DELETE,
  proxy as PUT,
};
