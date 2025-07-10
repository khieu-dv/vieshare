'use client';

import { ArrowRight } from "lucide-react";
import Link from "next/link";

import { Footer } from "~/ui/components/footer";
import { Header } from "~/ui/components/header";
import { Button } from "~/ui/primitives/button";
import { ContactButton } from "./components/contact-button";

export default function HomePage() {
  return (
    <>
      <Header />
      <main className="flex-1">
        <section className="py-12 md:py-16">
          <div className="container mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
            <div className="mb-8 flex flex-col items-center text-center">
              <h1 className="text-4xl font-bold tracking-tight sm:text-5xl md:text-6xl">
                Powerful FRP Port Mapping Manager
              </h1>
              <p className="mt-4 max-w-3xl text-lg text-muted-foreground">
                Easily manage your FRP (Fast Reverse Proxy) connections with a sleek and intuitive interface. Start, stop, and control port mappings in real-time for seamless remote access.
              </p>

              <div className="mt-10 flex justify-center">
                <Link href="/frps">
                  <Button size="lg" className="h-12 gap-1.5 px-8">
                    Manage Your FRP Now <ArrowRight className="h-4 w-4" />
                  </Button>
                </Link>
              </div>

              <ul className="mt-6 space-y-3 text-left text-muted-foreground text-base">
                <li>⚡ Connect or disconnect from FRP server with one click</li>
                <li>🛠 Add and remove port mappings dynamically</li>
                <li>🌐 Open remote apps with a single button</li>
                <li>🧩 Auto-generate proxy names and ports</li>
                <li>📡 Real-time connection status and diagnostics</li>
                <li>🔐 Secure configuration using token-based auth</li>
                <li>🧘 Clean, minimal UI powered by Tauri + Next.js</li>
              </ul>
            </div>
          </div>
        </section>
      </main>
      <Footer />
      <ContactButton />
    </>
  );
}
