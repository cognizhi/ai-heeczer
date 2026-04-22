import type { Metadata } from "next";
import "./globals.css";
import { Navigation } from "@/components/navigation";

export const metadata: Metadata = {
  title: "ai-heeczer Dashboard",
  description: "AI productivity scoring and analytics (PRD §21)",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-background font-sans antialiased">
        <div className="flex min-h-screen flex-col">
          <Navigation />
          <main className="flex-1 container mx-auto px-4 py-8">{children}</main>
        </div>
      </body>
    </html>
  );
}
