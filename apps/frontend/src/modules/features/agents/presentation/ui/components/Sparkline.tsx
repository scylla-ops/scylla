interface SparklineProps {
  data: number[];
  width?: number;
  height?: number;
  /** Accessible description of what the sparkline represents. */
  label?: string;
}

/** Minimal area sparkline — brand stroke with a soft brand fill. */
export const Sparkline = ({ data, width = 150, height = 20, label }: SparklineProps) => {
  const max = Math.max(1, ...data);
  const n = data.length;
  const stepX = n > 1 ? width / (n - 1) : width;
  const y = (v: number) => height - (v / max) * (height - 2) - 1;

  const points = data.map((v, i) => `${i * stepX},${y(v)}`);
  const linePath = points.length ? `M ${points.join(' L ')}` : '';
  const areaPath = points.length
    ? `M 0,${height} L ${points.join(' L ')} L ${width},${height} Z`
    : '';

  return (
    <svg
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio='none'
      role='img'
      aria-label={label ?? 'activity sparkline'}
    >
      {label && <title>{label}</title>}
      {areaPath && <path d={areaPath} fill='var(--success)' fillOpacity={0.12} />}
      {linePath && (
        <path
          d={linePath}
          fill='none'
          stroke='var(--success)'
          strokeWidth={1.5}
          strokeLinejoin='round'
          strokeLinecap='round'
        />
      )}
    </svg>
  );
};
