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
  medicineCount: number;
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

export interface BathSummary {
  totalCount: number;
  totalDurationMinutes: number;
  averageDurationMinutes: number;
}

// --- Date Range Presets ---

/**
 * Formats a Date as a YYYY-MM-DD string (date-only, no time component).
 * The API layer converts this to a full ISO 8601 datetime with timezone offset
 * before sending to the backend.
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
    medicineCount: feedings.filter((a) => a.medicine_added).length,
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

export function computeBathSummary(activities: ActivityResponse[]): BathSummary {
  const baths = activities.filter((a) => a.type === 'bath');
  const validBaths = baths.filter((a) => a.start_time != null && a.end_time != null);

  const totalDurationMinutes = validBaths.reduce((sum, a) => {
    const start = new Date(a.start_time!).getTime();
    const end = new Date(a.end_time!).getTime();
    return sum + (end - start) / 60000;
  }, 0);

  return {
    totalCount: baths.length,
    totalDurationMinutes,
    averageDurationMinutes: validBaths.length > 0 ? totalDurationMinutes / validBaths.length : 0,
  };
}

// --- Chart Data Transformations ---

export function transformFeedingChartData(activities: ActivityResponse[]): ChartDataPoint[] {
  return activities
    .filter((a) => a.type === 'feeding' && a.volume_ml != null)
    .map((a) => ({ label: a.timestamp, value: a.volume_ml! }))
    .sort((a, b) => a.label.localeCompare(b.label));
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
    .map(([label, value]) => ({ label, value: Number((value / 60).toFixed(2)) }));
}

export function computeWakeWindowSummary(activities: ActivityResponse[]): SleepSummary {
  const wakeWindows = activities.filter((a) => a.type === 'wake_window');
  const validWakeWindows = wakeWindows.filter((a) => a.start_time != null && a.end_time != null);

  const totalDurationMinutes = validWakeWindows.reduce((sum, a) => {
    const start = new Date(a.start_time!).getTime();
    const end = new Date(a.end_time!).getTime();
    return sum + (end - start) / 60000;
  }, 0);

  return {
    totalCount: wakeWindows.length,
    totalDurationMinutes,
    averageDurationMinutes:
      validWakeWindows.length > 0 ? totalDurationMinutes / validWakeWindows.length : 0,
  };
}

export function transformWakeWindowChartData(activities: ActivityResponse[]): ChartDataPoint[] {
  const validWakeWindows = activities.filter(
    (a) => a.type === 'wake_window' && a.start_time != null && a.end_time != null
  );

  const byDay = new Map<string, number>();
  for (const a of validWakeWindows) {
    const day = a.start_time!.slice(0, 10);
    const durationMin =
      (new Date(a.end_time!).getTime() - new Date(a.start_time!).getTime()) / 60000;
    byDay.set(day, (byDay.get(day) ?? 0) + durationMin);
  }

  return Array.from(byDay.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([label, value]) => ({ label, value: Number((value / 60).toFixed(2)) }));
}

export function transformBathChartData(activities: ActivityResponse[]): ChartDataPoint[] {
  const validBaths = activities.filter(
    (a) => a.type === 'bath' && a.start_time != null && a.end_time != null
  );

  const byDay = new Map<string, number>();
  for (const a of validBaths) {
    const day = a.start_time!.slice(0, 10);
    byDay.set(day, (byDay.get(day) ?? 0) + 1);
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

export interface DiaperStackedDataPoint {
  label: string;
  wet: number;
  dirty: number;
  both: number;
}

/**
 * Groups diaper change activities by calendar day with separate counts for
 * wet, dirty, and both — suitable for a stacked bar chart.
 */
export function transformDiaperStackedChartData(
  activities: ActivityResponse[]
): DiaperStackedDataPoint[] {
  const diapers = activities.filter((a) => a.type === 'diaper_change');

  const byDay = new Map<string, { wet: number; dirty: number; both: number }>();
  for (const a of diapers) {
    const day = a.timestamp.slice(0, 10);
    const existing = byDay.get(day) ?? { wet: 0, dirty: 0, both: 0 };
    if (a.contents === 'wet') existing.wet += 1;
    else if (a.contents === 'dirty') existing.dirty += 1;
    else if (a.contents === 'both') existing.both += 1;
    byDay.set(day, existing);
  }

  return Array.from(byDay.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([label, counts]) => ({ label, ...counts }));
}

export function transformPumpingChartData(activities: ActivityResponse[]): ChartDataPoint[] {
  return activities
    .filter((a) => a.type === 'pumping' && a.volume_ml != null)
    .map((a) => ({ label: a.timestamp, value: a.volume_ml! }))
    .sort((a, b) => a.label.localeCompare(b.label));
}

// --- Composite (Bar + Line) Transformations ---

export interface CompositeDataPoint {
  label: string;
  total: number;
  average: number;
  totalOz: number;
  averageOz: number;
}

// --- Trend (Running Average) Transformations ---

export type TrendWindow = '1h' | '6h' | '1d';

const TREND_WINDOW_MS: Record<TrendWindow, number> = {
  '1h': 60 * 60 * 1000,
  '6h': 6 * 60 * 60 * 1000,
  '1d': 24 * 60 * 60 * 1000,
};

/**
 * Buckets volume-based activities into fixed time windows and computes the
 * average volume per window. Useful for spotting trends over longer ranges
 * instead of plotting every individual data point.
 */
export function transformVolumeTrendData(
  activities: ActivityResponse[],
  activityType: 'feeding' | 'pumping',
  window: TrendWindow
): ChartDataPoint[] {
  const filtered = activities
    .filter((a) => a.type === activityType && a.volume_ml != null)
    .sort((a, b) => a.timestamp.localeCompare(b.timestamp));

  if (filtered.length === 0) return [];

  const windowMs = TREND_WINDOW_MS[window];
  const buckets = new Map<number, { sum: number; count: number }>();

  for (const a of filtered) {
    const ts = new Date(a.timestamp).getTime();
    const bucketKey = Math.floor(ts / windowMs) * windowMs;
    const existing = buckets.get(bucketKey);
    if (existing) {
      existing.sum += a.volume_ml!;
      existing.count += 1;
    } else {
      buckets.set(bucketKey, { sum: a.volume_ml!, count: 1 });
    }
  }

  const formatLabel = (ms: number): string => {
    const d = new Date(ms);
    if (window === '1d') {
      return formatDateOnly(d);
    }
    // For hourly windows, show date + time
    const date = formatDateOnly(d);
    const h = String(d.getHours()).padStart(2, '0');
    const m = String(d.getMinutes()).padStart(2, '0');
    return `${date} ${h}:${m}`;
  };

  return Array.from(buckets.entries())
    .sort(([a], [b]) => a - b)
    .map(([bucketKey, { sum, count }]) => ({
      label: formatLabel(bucketKey),
      value: Math.round(sum / count),
    }));
}

/**
 * Buckets volume-based activities into fixed time windows and returns both
 * the raw total and the average volume per window — suitable for a composite
 * bar (total) + line (average) chart.
 */
export function transformVolumeCompositeData(
  activities: ActivityResponse[],
  activityType: 'feeding' | 'pumping',
  window: TrendWindow
): CompositeDataPoint[] {
  const filtered = activities
    .filter((a) => a.type === activityType && a.volume_ml != null)
    .sort((a, b) => a.timestamp.localeCompare(b.timestamp));

  if (filtered.length === 0) return [];

  const windowMs = TREND_WINDOW_MS[window];
  const buckets = new Map<number, { sum: number; count: number }>();

  for (const a of filtered) {
    const ts = new Date(a.timestamp).getTime();
    const bucketKey = Math.floor(ts / windowMs) * windowMs;
    const existing = buckets.get(bucketKey);
    if (existing) {
      existing.sum += a.volume_ml!;
      existing.count += 1;
    } else {
      buckets.set(bucketKey, { sum: a.volume_ml!, count: 1 });
    }
  }

  const formatLabel = (ms: number): string => {
    const d = new Date(ms);
    if (window === '1d') {
      return formatDateOnly(d);
    }
    const date = formatDateOnly(d);
    const h = String(d.getHours()).padStart(2, '0');
    const m = String(d.getMinutes()).padStart(2, '0');
    return `${date} ${h}:${m}`;
  };

  return Array.from(buckets.entries())
    .sort(([a], [b]) => a - b)
    .map(([bucketKey, { sum, count }]) => ({
      label: formatLabel(bucketKey),
      total: Math.round(sum),
      average: Math.round(sum / count),
      totalOz: Math.round((sum / 29.574) * 10) / 10,
      averageOz: Math.round((sum / count / 29.574) * 10) / 10,
    }));
}
