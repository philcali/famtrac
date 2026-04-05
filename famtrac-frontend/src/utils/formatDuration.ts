/**
 * Formats a duration given in total minutes into "Xh Ym" format.
 * @param totalMinutes - Non-negative integer representing total minutes
 * @returns Formatted string like "2h 30m", "0h 45m", "0h 0m"
 */
export function formatDuration(totalMinutes: number): string {
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  return `${hours}h ${minutes}m`;
}
