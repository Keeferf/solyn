interface SkeletonProps {
  className?: string;
}

export const Skeleton = ({ className = "" }: SkeletonProps) => (
  <div aria-hidden="true" className={`shimmer rounded bg-white/5 ${className}`} />
);
