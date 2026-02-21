import { LANDING_SECTION_IDS } from "./sectionIds";
import { BentoCard, PillTag } from "./shared";

export function HowItWorks() {
  return (
    <section id={LANDING_SECTION_IDS.howItWorks} className="mx-auto max-w-7xl px-4 py-section sm:px-6 lg:px-8">
      <PillTag tone="neutral">How It Works</PillTag>
      <h2 className="mt-3 text-heading-xl text-text-primary">Three steps from idea to live token</h2>
      <div className="mt-8 grid gap-4 md:grid-cols-2">
        <BentoCard title="1. Connect wallet" description="Securely connect Freighter and choose your target network." className="md:col-span-1" />
        <BentoCard title="2. Configure token" description="Set code, issuer options, and metadata with instant field validation." className="md:col-span-1" />
        <BentoCard title="3. Review and deploy" description="Confirm fees, sign once, and publish the asset to Stellar." className="md:col-span-2" />
      </div>
    </section>
  );
}
