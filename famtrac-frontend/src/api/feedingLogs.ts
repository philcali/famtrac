import { ApiClient } from './client';
import type { ApiResponse } from './client';
import type {
  CreateFeedingLogRequest,
  UpdateFeedingLogRequest,
  FeedingLogResponse,
  FeedingLogListResponse,
} from './types';

/**
 * Get all feeding logs for a specific dependent
 */
export async function getFeedingLogs(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  date?: string,
  limit?: number,
  nextToken?: string
): Promise<ApiResponse<FeedingLogListResponse>> {
  const params = new URLSearchParams();
  if (date) params.append('date', date);
  if (limit) params.append('limit', String(limit));
  if (nextToken) params.append('next_token', nextToken);
  const query = params.toString();
  const path = query
    ? `/families/${familyId}/dependents/${dependentId}/feeding-logs?${query}`
    : `/families/${familyId}/dependents/${dependentId}/feeding-logs`;
  return client.get<FeedingLogListResponse>(path);
}

/**
 * Get a single feeding log by ID
 */
export async function getFeedingLog(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  feedingLogId: string
): Promise<ApiResponse<FeedingLogResponse>> {
  return client.get<FeedingLogResponse>(
    `/families/${familyId}/dependents/${dependentId}/feeding-logs/${feedingLogId}`
  );
}

/**
 * Create a new feeding log
 */
export async function createFeedingLog(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  request: CreateFeedingLogRequest
): Promise<ApiResponse<FeedingLogResponse>> {
  return client.post<FeedingLogResponse>(
    `/families/${familyId}/dependents/${dependentId}/feeding-logs`,
    request
  );
}

/**
 * Update an existing feeding log
 */
export async function updateFeedingLog(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  feedingLogId: string,
  request: UpdateFeedingLogRequest
): Promise<ApiResponse<FeedingLogResponse>> {
  return client.put<FeedingLogResponse>(
    `/families/${familyId}/dependents/${dependentId}/feeding-logs/${feedingLogId}`,
    request
  );
}

/**
 * Delete a feeding log
 */
export async function deleteFeedingLog(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  feedingLogId: string
): Promise<ApiResponse<void>> {
  return client.delete<void>(
    `/families/${familyId}/dependents/${dependentId}/feeding-logs/${feedingLogId}`
  );
}
