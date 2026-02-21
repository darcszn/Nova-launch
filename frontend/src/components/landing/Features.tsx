import { LANDING_SECTION_IDS } from "./sectionIds";
import { FeatureCard, PillTag } from "./shared";

export function Features() {
  return (
    <section id={LANDING_SECTION_IDS.features} className="mx-auto max-w-7xl px-4 py-section sm:px-6 lg:px-8">
      <PillTag tone="neutral">Features</PillTag>
      <h2 className="mt-3 text-heading-xl text-text-primary">Everything needed for token deployment</h2>
      <div className="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <FeatureCard title="Wallet-native setup" description="Connect Freighter and deploy directly from your account with no private key exports." />
        <FeatureCard title="Metadata workflow" description="Attach logos, descriptions, and links in one guided flow with validation built in." />
        <FeatureCard title="Transaction visibility" description="Track deployment status, outcomes, and records with clear transaction history." />
      </div>
    </section>
  );
}
