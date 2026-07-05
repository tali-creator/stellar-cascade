import type { Metadata } from "next";
import "./globals.css";
export const metadata: Metadata = {
  title: "Cascade — Funding that flows through the dependency tree",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link
          href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600;700&family=IBM+Plex+Sans:wght@400;500;600;700&display=swap"
          rel="stylesheet"
        />
      </head>
      <body className="bg-[#0A1410] font-['IBM_Plex_Sans']">{children}</body>
    </html>
  );
}
