import { LANDING_SECTION_IDS } from "./sectionIds";
import { PillTag } from "./shared";

export function Hero() {
  return (
    <section id={LANDING_SECTION_IDS.hero} className="mx-auto grid max-w-7xl gap-10 px-4 pb-section-sm pt-section-sm sm:px-6 lg:grid-cols-2 lg:px-8 lg:pt-section">
      <div>
        <PillTag>Stellar Infrastructure</PillTag>
        <h1 className="mt-4 text-display-lg tracking-tight text-text-primary lg:text-display-xl">
          Launch production-ready Stellar assets in minutes.
        </h1>
        <p className="mt-5 max-w-xl text-base leading-7 text-text-secondary sm:text-lg">
          Nova helps teams deploy tokens, manage metadata, and monitor transactions with a developer-first workflow.
        </p>
        <div className="mt-8 flex flex-wrap gap-3">
          <a
            href="/deploy"
            className="inline-flex h-11 items-center rounded-card bg-primary px-5 text-sm font-semibold text-text-primary transition hover:opacity-90 hover:shadow-glow-red"
          >
            Start deploying
          </a>
          <a
            href={`#${LANDING_SECTION_IDS.howItWorks}`}
            data-scroll-link="true"
            className="inline-flex h-11 items-center rounded-card border border-border-medium bg-background-elevated px-5 text-sm font-semibold text-text-primary transition hover:border-primary/60"
          >
            See how it works
          </a>
        </div>
      </div>

      <nav aria-label="Section navigation" className="rounded-card border border-border-medium bg-background-card p-5 shadow-card-hover">
        <p className="text-sm font-semibold uppercase tracking-wide text-text-muted">Jump to section</p>
        <ul className="mt-3 space-y-2 text-sm">
          <li>
            <a className="text-text-secondary hover:text-primary" href={`#${LANDING_SECTION_IDS.hero}`} data-scroll-link="true">Overview</a>
          </li>
          <li>
            <a className="text-text-secondary hover:text-primary" href={`#${LANDING_SECTION_IDS.features}`} data-scroll-link="true">Features</a>
          </li>
          <li>
            <a className="text-text-secondary hover:text-primary" href={`#${LANDING_SECTION_IDS.howItWorks}`} data-scroll-link="true">How it works</a>
          </li>
          <li>
            <a className="text-text-secondary hover:text-primary" href={`#${LANDING_SECTION_IDS.faq}`} data-scroll-link="true">FAQ</a>
          </li>
          <li>
            <a className="text-text-secondary hover:text-primary" href={`#${LANDING_SECTION_IDS.footer}`} data-scroll-link="true">Contact</a>
          </li>
        </ul>
      </nav>
    </section>
  );
}
