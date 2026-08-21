export interface SkeletonCardProps {
  count?: number;
}

/**
 * SkeletonCard component - Placeholder UI for loading cards
 * - Displays skeleton loading UI for data fetching (Requirement 19.3)
 */
export function SkeletonCard({ count = 1 }: SkeletonCardProps) {
  return (
    <>
      {Array.from({ length: count }).map((_, index) => (
        <div key={index} className="mb-3 p-4 bg-white rounded-xl border border-gray-100 shadow-sm">
          <div className="skeleton skeleton-title mb-3 h-4 w-3/4"></div>
          <div className="skeleton skeleton-text mb-2 h-3 w-full"></div>
          <div className="skeleton skeleton-text mb-3 h-3 w-4/5"></div>
          <div className="flex gap-2">
            <div className="skeleton w-[60px] h-8"></div>
            <div className="skeleton w-[60px] h-8"></div>
            <div className="skeleton w-[60px] h-8"></div>
          </div>
        </div>
      ))}
    </>
  );
}
