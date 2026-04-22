import type { NextConfig } from "next";

// Parametrise the ingest origin so connect-src matches deployment realities.
const ingestOrigin =
  process.env["NEXT_PUBLIC_INGEST_URL"] ?? "http://localhost:8080";

const nextConfig: NextConfig = {
  // Standalone output for container deployment (plan 0010).
  output: "standalone",
  // Strict mode for React 19.
  reactStrictMode: true,
  // Security headers (plan 0010, ADR-0008).
  async headers() {
    return [
      {
        source: "/(.*)",
        headers: [
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "X-Frame-Options", value: "DENY" },
          { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
          {
            key: "Strict-Transport-Security",
            value: "max-age=31536000; includeSubDomains",
          },
          {
            key: "Content-Security-Policy",
            value: [
              "default-src 'self'",
              // Next.js 15 requires 'unsafe-inline' for inline styles and
              // event handlers injected by the framework. 'unsafe-eval' is
              // removed from production builds.
              `script-src 'self'${process.env.NODE_ENV === "development" ? " 'unsafe-eval'" : ""} 'unsafe-inline'`,
              "style-src 'self' 'unsafe-inline'",
              "img-src 'self' data: blob:",
              `connect-src 'self' ${ingestOrigin}`,
              "worker-src 'self' blob:",
              "font-src 'self'",
              "object-src 'none'",
              "base-uri 'self'",
              "form-action 'self'",
            ].join("; "),
          },
        ],
      },
    ];
  },
};

export default nextConfig;
