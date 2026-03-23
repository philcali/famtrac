import type { ActivityResponse } from '../api/types';

// --- Types ---

export type TimeRangePreset = 'today' | 'week' | 'month';

export interface ChartDataPoint {
  label: string;
  value: number;
}

export interface FeedingSummary {
  totalCount: number;
  totalVolumeMl: number;
  averageVolumeMl: number;
}

export interface SleepSummary {
  totalCount: number;
  totalDurationMinutes: number;
  averageDurationMinutes: number;
}

export interface DiaperSummary {
  totalCount: number;
  wetCount: number;
  dirtyCount: number;
  bothCount: number;
}

export interface PumpingSummary {
  totalCount: number;
  totalVolumeMl: number;
  averageVolumeMl: number;
}

// --- Date Range Presets ---

/**
 * Formats a Date as a YYYY-MM-DD string (date-only, no time component).
 * The backend expects NaiveDate (chrono) which parses YYYY-MM-DD only.
 */
function formatDateOnly(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

export function getPresetDateRange(preset: TimeRangePreset): {
  startDate: string;
  endDate: string;
} {
  const now = new Date();
  const endDate = formatDateOnly(now);

  switch (preset) {
    case 'today': {
      return { startDate: formatDateOnly(now), endDate };
    }
    case 'week': {
      const day = now.getDay();
      // Monday = 1, Sunday = 0 → offset so Monday is start of week
      const diffToMonday = day === 0 ? 6 : day - 1;
      const start = new Date(now.getFullYear(), now.getMonth(), now.getDate() - diffToMonday);
      return { startDate: formatDateOnly(start), endDate };
    }
    case 'month': {
      const start = new Date(now.getFullYear(), now.getMonth(), 1);
      return { startDate: formatDateOnly(start), endDate };
    }
  }
}

// --- Summary Computations ---

export function computeFeedingSummary(activities: ActivityResponse[]): FeedingSummary {
  const feedings = activities.filter((a) => a.type === 'feeding');
  const withVolume = feedings.filter((a) => a.volume_ml != null);
  const totalVolumeMl = withVolume.reduce((sum, a) => sum + a.volume_ml!, 0);

  return {
    totalCount: feedings.length,
    totalVolumeMl,
    averageVolumeMl: withVolume.length > 0 ? totalVolumeMl / withVolume.length : 0,
  };
}

export function computeSleepSummary(activities: ActivityResponse[]): SleepSummary {
  const sleeps = activities.filter((a) => a.type === 'sleep');
  const validSleeps = sleeps.filter((a) => a.start_time != null && a.end_time != null);

  const totalDurationMinutes = validSleeps.reduce((sum, a) => {
    const start = new Date(a.start_time!).getTime();
    const end = new Date(a.end_time!).getTime();
    return sum + (end - start) / 60000;
  }, 0);

  return {
    totalCount: sleeps.length,
    totalDurationMinutes,
    averageDurationMinutes: validSleeps.length > 0 ? totalDurationMinutes / validSleeps.length : 0,
  };
}

export function computeDiaperSummary(activities: ActivityResponse[]): DiaperSummary {
  const diapers = activities.filter((a) => a.type === 'diaper_change');

  return {
    totalCount: diapers.length,
    wetCount: diapers.filter((a) => a.contents === 'wet').length,
    dirtyCount: diapers.filter((a) => a.contents === 'dirty').length,
    bothCount: diapers.filter((a) => a.contents === 'both').length,
  };
}

export function computePumpingSummary(activities: ActivityResponse[]): PumpingSummary {
  const pumpings = activities.filter((a) => a.type === 'pumping');
  const withVolume = pumpings.filter((a) => a.volume_ml != null);
  const totalVolumeMl = withVolume.reduce((sum, a) => sum + a.volume_ml!, 0);

  return {
    totalCount: pumpings.length,
    totalVolumeMl,
    averageVolumeMl: withVolume.length > 0 ? totalVolumeMl / withVolume.length : 0,
  };
}

// --- Chart Data Transformations ---

export function transformFeedingChartData(activities: ActivityResponse[]): ChartDataPoint[] {
  return activities
    .filter((a) => a.type === 'feeding' && a.volume_ml != null)
    .map((a) => ({ label: a.timestamp, value: a.volume_ml! }));
}

export function transformSleepChartData(activities: ActivityResponse[]): ChartDataPoint[] {
  const validSleeps = activities.filter(
    (a) => a.type === 'sleep' && a.start_time != null && a.end_time != null
  );

  const byDay = new Map<string, number>();
  for (const a of validSleeps) {
    const day = a.start_time!.slice(0, 10); // YYYY-MM-DD
    const durationMin =
      (new Date(a.end_time!).getTime() - new Date(a.start_time!).getTime()) / 60000;
    byDay.set(day, (byDay.get(day) ?? 0) + durationMin);
  }

  return Array.from(byDay.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([label, value]) => ({ label, value }));
}

export function transformDiaperChartData(activities: ActivityResponse[]): ChartDataPoint[] {
  const diapers = activities.filter((a) => a.type === 'diaper_change');

  const byDay = new Map<string, number>();
  for (const a of diapers) {
    const day = a.timestamp.slice(0, 10); // YYYY-MM-DD
    byDay.set(day, (byDay.get(day) ?? 0) + 1);
  }

  return Array.from(byDay.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([label, value]) => ({ label, value }));
}

export function transformPumpingChartData(activities: ActivityResponse[]): ChartDataPoint[] {
  return activities
    .filter((a) => a.type === 'pumping' && a.volume_ml != null)
    .map((a) => ({ label: a.timestamp, value: a.volume_ml! }));
}
