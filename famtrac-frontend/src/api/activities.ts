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
 * Get all activities for a specific dependent
 * Supports optional filtering by date range and activity type
 */
export async function getActivities(
  client: ApiClient,
  dependentId: string,
  options?: {
    startDate?: string; // ISO 8601 format
    endDate?: string; // ISO 8601 format
    activityType?: ActivityType;
  }
): Promise<ApiResponse<ActivityListResponse>> {
  let path = `/dependents/${dependentId}/activities`;

  // Build query parameters for filtering
  const params = new URLSearchParams();
  if (options?.startDate) {
    params.append('start_date', options.startDate);
  }
  if (options?.endDate) {
    params.append('end_date', options.endDate);
  }
  if (options?.activityType) {
    params.append('activity_type', options.activityType);
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
  activityId: string
): Promise<ApiResponse<ActivityResponse>> {
  return client.get<ActivityResponse>(`/activities/${activityId}`);
}

/**
 * Create a new activity
 */
export async function createActivity(
  client: ApiClient,
  request: CreateActivityRequest
): Promise<ApiResponse<ActivityResponse>> {
  return client.post<ActivityResponse>('/activities', request);
}

/**
 * Update an existing activity
 */
export async function updateActivity(
  client: ApiClient,
  activityId: string,
  request: UpdateActivityRequest
): Promise<ApiResponse<ActivityResponse>> {
  return client.put<ActivityResponse>(`/activities/${activityId}`, request);
}

/**
 * Delete an activity
 */
export async function deleteActivity(
  client: ApiClient,
  activityId: string
): Promise<ApiResponse<void>> {
  return client.delete<void>(`/activities/${activityId}`);
}
