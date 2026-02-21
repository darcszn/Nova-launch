interface BentoCardProps {
  title: string;
  description: string;
  className?: string;
}

export function BentoCard({ title, description, className = "" }: BentoCardProps) {
  return (
    <article className={`rounded-card border border-border-medium bg-background-card p-6 shadow-card-hover ${className}`}>
      <h3 className="text-lg font-semibold text-text-primary">{title}</h3>
      <p className="mt-2 text-sm leading-6 text-text-secondary">{description}</p>
    </article>
  );
}
