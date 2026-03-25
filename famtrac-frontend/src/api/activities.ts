import { ApiClient } from './client';
import type { ApiResponse } from './client';
import type {
  CreateActivityRequest,
  UpdateActivityRequest,
  ActivityResponse,
  ActivityListResponse,
  ActivityType,
} from './types';

/**
 * Converts a YYYY-MM-DD date string to an ISO 8601 datetime string
 * at the start of that day in the user's local timezone.
 * e.g. "2024-01-15" in UTC-5 → "2024-01-15T00:00:00-05:00"
 */
function toLocalStartOfDay(dateStr: string): string {
  const [y, m, d] = dateStr.split('-').map(Number);
  const local = new Date(y, m - 1, d, 0, 0, 0);
  return toISOWithOffset(local);
}

/**
 * Converts a YYYY-MM-DD date string to an ISO 8601 datetime string
 * at the end of that day in the user's local timezone.
 * e.g. "2024-01-15" in UTC-5 → "2024-01-15T23:59:59-05:00"
 */
function toLocalEndOfDay(dateStr: string): string {
  const [y, m, d] = dateStr.split('-').map(Number);
  const local = new Date(y, m - 1, d, 23, 59, 59);
  return toISOWithOffset(local);
}

/**
 * Formats a Date as an ISO 8601 string with the local timezone offset.
 * e.g. "2024-01-15T00:00:00-05:00"
 */
function toISOWithOffset(date: Date): string {
  const off = -date.getTimezoneOffset();
  const sign = off >= 0 ? '+' : '-';
  const absOff = Math.abs(off);
  const hh = String(Math.floor(absOff / 60)).padStart(2, '0');
  const mm = String(absOff % 60).padStart(2, '0');
  const pad = (n: number) => String(n).padStart(2, '0');
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}` +
    `T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}` +
    `${sign}${hh}:${mm}`
  );
}

/**
 * Get all activities for a specific dependent
 * Supports optional filtering by date range and activity type
 */
export async function getActivities(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  options?: {
    startDate?: string; // YYYY-MM-DD date format
    endDate?: string; // YYYY-MM-DD date format
    activityType?: ActivityType;
    limit?: number;
    next_token?: string;
  }
): Promise<ApiResponse<ActivityListResponse>> {
  let path = `/families/${familyId}/dependents/${dependentId}/activities`;

  // Build query parameters for filtering
  const params = new URLSearchParams();
  if (options?.startDate) {
    params.append('start_date', toLocalStartOfDay(options.startDate));
  }
  if (options?.endDate) {
    params.append('end_date', toLocalEndOfDay(options.endDate));
  }
  if (options?.activityType) {
    params.append('activity_type', options.activityType);
  }
  if (options?.limit) {
    params.append('limit', String(options.limit));
  }
  if (options?.next_token) {
    params.append('next_token', options.next_token);
  }

  // Append query string if there are any parameters
  const queryString = params.toString();
  if (queryString) {
    path += `?${queryString}`;
  }

  return client.get<ActivityListResponse>(path);
}

/**
 * Get a single activity by ID
 */
export async function getActivity(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  activityId: string
): Promise<ApiResponse<ActivityResponse>> {
  return client.get<ActivityResponse>(
    `/families/${familyId}/dependents/${dependentId}/activities/${activityId}`
  );
}

/**
 * Create a new activity
 */
export async function createActivity(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  request: CreateActivityRequest
): Promise<ApiResponse<ActivityResponse>> {
  return client.post<ActivityResponse>(
    `/families/${familyId}/dependents/${dependentId}/activities`,
    request
  );
}

/**
 * Update an existing activity
 */
export async function updateActivity(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  activityId: string,
  request: UpdateActivityRequest
): Promise<ApiResponse<ActivityResponse>> {
  return client.put<ActivityResponse>(
    `/families/${familyId}/dependents/${dependentId}/activities/${activityId}`,
    request
  );
}

/**
 * Delete an activity
 */
export async function deleteActivity(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  activityId: string
): Promise<ApiResponse<void>> {
  return client.delete<void>(
    `/families/${familyId}/dependents/${dependentId}/activities/${activityId}`
  );
}
