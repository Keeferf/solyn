interface RingSpinnerProps {
  size?: number;
  className?: string;
}

export const RingSpinner = ({
  size = 12,
  className = "border-2 border-white/20 border-t-white/60",
}: RingSpinnerProps) => (
  <span
    className={`inline-block rounded-full animate-spin shrink-0 ${className}`}
    style={{ width: size, height: size }}
  />
);
