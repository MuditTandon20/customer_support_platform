import type { Metadata } from "next";
import "./globals.css";
import "./workspace.css";
import "./stage1.css";

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
        <div className="beta-banner" role="note">
          <strong>ASTER STAGE 1 ? TESTING BETA</strong>
          <span>
            Non-commercial ? Synthetic data only ? Extractive assistance,
            human-reviewed
          </span>
        </div>
        {children}
      </body>
    </html>
  );
}
