const VARIANT_COLORS: Record<string, string> = {
  primary: 'bg-blue-600',
  secondary: 'bg-gray-600',
  success: 'bg-green-600',
  danger: 'bg-red-600',
  warning: 'bg-yellow-500',
  info: 'bg-sky-500',
  light: 'bg-gray-100',
  dark: 'bg-gray-800',
};

export interface ActivitySummaryCardProps {
  title: string;
  metrics: { label: string; value: string }[];
  variant: string;
}

/**
 * ActivitySummaryCard displays aggregated metrics for an activity type.
 * Renders a colored header and a list of label/value pairs.
 */
export function ActivitySummaryCard({ title, metrics, variant }: ActivitySummaryCardProps) {
  const headerColor = VARIANT_COLORS[variant] || 'bg-gray-600';
  return (
    <div className="mb-3">
      <div className={`p-4 ${headerColor} text-white rounded-t-xl`}>
        <h3 className="text-base font-semibold">{title}</h3>
      </div>
      <div className="bg-white border border-t-0 border-gray-100 rounded-b-xl divide-y divide-gray-100">
        {metrics.map((metric) => (
          <div key={metric.label} className="flex justify-between items-center py-2 px-4 text-sm">
            <span>{metric.label}</span>
            <strong>{metric.value}</strong>
          </div>
        ))}
      </div>
    </div>
  );
}
