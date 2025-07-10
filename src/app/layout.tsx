import { SpeedInsights } from "@vercel/speed-insights/next";
import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { ThemeProvider } from "~/ui/components/theme-provider";
import Providers from "./components/providers/Providers";
import I18nProvider from "./components/providers/i18nProvider";


const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "VieShare - Advanced Programming Channel",
  description:
    "Welcome to the advanced programming channel! Here, I share practical knowledge and detailed tutorials on modern languages and frameworks such as Go, Rust, ReactJS, and Next.js. Suitable for both beginners and developers looking to enhance their skills.",
  keywords: [
    "programming",
    "VieShare",
    "Go",
    "Rust",
    "ReactJS",
    "Next.js",
    "learn programming",
    "coding tutorials",
    "frontend",
    "backend",
    "fullstack development",
  ],
  authors: [{ name: "VieShare" }],
  creator: "VieShare",
  generator: "Next.js",
  metadataBase: new URL("https://vieshare.com"),
  openGraph: {
    title: "VieShare - Advanced Programming Channel",
    description:
      "Practical insights & detailed tutorials on Go, Rust, ReactJS, and Next.js. Perfect for beginners and experienced developers alike.",
    url: "https://vieshare.com",
    siteName: "VieShare",
    images: [
      {
        url: "https://vieshare.com/og-image.jpg",
        width: 1200,
        height: 630,
        alt: "VieShare - Advanced Programming Channel",
      },
    ],
    locale: "en_US",
    type: "website",
  },
};


export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        {/* Google Tag (gtag.js) */}
        <script async src="https://www.googletagmanager.com/gtag/js?id=G-1PNDJK7RXF"></script>
        <script
          dangerouslySetInnerHTML={{
            __html: `
              window.dataLayer = window.dataLayer || [];
              function gtag(){dataLayer.push(arguments);}
              gtag('js', new Date());
              gtag('config', 'G-1PNDJK7RXF');
            `,
          }}
        />
      </head>
      <body className={`${geistSans.variable} ${geistMono.variable} antialiased`}>
        <ThemeProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange
        >
          <I18nProvider>
            <Providers>
              {children}
              {/* <DevToolsDetector /> */}
            </Providers>
          </I18nProvider>
        </ThemeProvider>
        <SpeedInsights />
      </body>
    </html>
  );
}
