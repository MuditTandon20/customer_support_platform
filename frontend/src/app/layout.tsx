import type { Metadata } from "next";
import "./globals.css";
import "./workspace.css";

export const metadata: Metadata = {
  title: "Aster Support OS",
  description: "Evidence-first support with human control",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="bg-[#0B0F14] font-body text-[#E8EDF2] antialiased">
        {children}
      </body>
    </html>
  );
}
