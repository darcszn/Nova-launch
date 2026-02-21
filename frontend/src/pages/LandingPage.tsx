import { useCallback, useEffect, useRef } from "react";
import { FAQ, Features, Footer, Hero, HowItWorks } from "../components/landing";
import { LANDING_SCROLL_ORDER } from "../components/landing/sectionIds";
import {
  PWAInstallButton,
  PWAConnectionStatus,
} from "../components/PWA";
import { NetworkToggle, WalletInfo } from "../components/WalletConnect";
import { Button } from "../components/UI";
import type { WalletState } from "../types";

const SCROLL_DURATION_MS = 700;

interface LandingPageProps {
  network: "testnet" | "mainnet";
  setNetwork: (network: "testnet" | "mainnet") => void;
  wallet: WalletState;
  connect: () => Promise<void>;
  disconnect: () => void;
  isConnecting: boolean;
}

function easeInOutCubic(progress: number): number {
  return progress < 0.5
    ? 4 * progress * progress * progress
    : 1 - Math.pow(-2 * progress + 2, 3) / 2;
}

export default function LandingPage({
  network,
  setNetwork,
  wallet,
  connect,
  disconnect,
  isConnecting,
}: LandingPageProps) {
  const animationRef = useRef<number | null>(null);

  const stopCurrentScroll = useCallback(() => {
    if (animationRef.current !== null) {
      window.cancelAnimationFrame(animationRef.current);
      animationRef.current = null;
    }
  }, []);

  const scrollToSection = useCallback((sectionId: string, shouldPushState = true) => {
    const target = document.getElementById(sectionId);
    if (!target) {
      return;
    }

    stopCurrentScroll();

    const startY = window.scrollY;
    const targetY = window.scrollY + target.getBoundingClientRect().top;
    const distance = targetY - startY;
    const start = performance.now();

    const animate = (now: number) => {
      const elapsed = now - start;
      const progress = Math.min(elapsed / SCROLL_DURATION_MS, 1);
      const eased = easeInOutCubic(progress);

      window.scrollTo(0, startY + distance * eased);

      if (progress < 1) {
        animationRef.current = window.requestAnimationFrame(animate);
      } else {
        animationRef.current = null;
      }
    };

    animationRef.current = window.requestAnimationFrame(animate);

    const nextUrl = `${window.location.pathname}#${sectionId}`;
    if (shouldPushState) {
      window.history.pushState(null, "", nextUrl);
    } else {
      window.history.replaceState(null, "", nextUrl);
    }
  }, [stopCurrentScroll]);

  useEffect(() => {
    const handleClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      const anchor = target?.closest('a[data-scroll-link="true"]') as HTMLAnchorElement | null;

      if (!anchor) {
        return;
      }

      const hash = anchor.getAttribute("href")?.split("#")[1];
      if (!hash) {
        return;
      }

      event.preventDefault();
      scrollToSection(hash);
    };

    document.addEventListener("click", handleClick);
    return () => {
      document.removeEventListener("click", handleClick);
      stopCurrentScroll();
    };
  }, [scrollToSection, stopCurrentScroll]);

  useEffect(() => {
    const initialHash = window.location.hash.replace("#", "");
    if (initialHash && LANDING_SCROLL_ORDER.includes(initialHash as (typeof LANDING_SCROLL_ORDER)[number])) {
      const timer = window.setTimeout(() => {
        scrollToSection(initialHash, false);
      }, 0);

      return () => window.clearTimeout(timer);
    }

    return undefined;
  }, [scrollToSection]);

  return (
    <main className="landing-page dark bg-background-dark text-left text-text-primary">
      <header className="border-b border-border-subtle bg-background-dark/80 backdrop-blur">
        <div className="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-3 px-4 py-4 sm:px-6 lg:px-8">
          <p className="text-sm font-semibold tracking-wide text-text-secondary">NovaLaunch</p>
          <div className="flex flex-wrap items-center gap-2 sm:gap-3">
            <PWAConnectionStatus />
            <PWAInstallButton />
            <NetworkToggle network={network} onNetworkChange={setNetwork} />
            {wallet.connected && wallet.address ? (
              <WalletInfo wallet={wallet} onDisconnect={disconnect} />
            ) : (
              <Button size="sm" onClick={() => void connect()} loading={isConnecting}>
                Connect Wallet
              </Button>
            )}
          </div>
        </div>
      </header>
      <Hero />
      <Features />
      <HowItWorks />
      <FAQ />
      <Footer />
    </main>
  );
}
