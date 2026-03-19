import { ApiClient } from './client';
import type { ApiResponse } from './client';
import type {
  CreateShareRequest,
  UpdateShareRequest,
  ShareResponse,
  ShareListResponse,
} from './types';

/**
 * Create a new share (invite a user by email)
 */
export async function createShare(
  client: ApiClient,
  familyId: string,
  request: CreateShareRequest
): Promise<ApiResponse<ShareResponse>> {
  return client.post<ShareResponse>(`/families/${familyId}/shares`, request);
}

/**
 * Get shares for a specific family with optional pagination
 */
export async function getShares(
  client: ApiClient,
  familyId: string,
  options?: { limit?: number; next_token?: string }
): Promise<ApiResponse<ShareListResponse>> {
  let path = `/families/${familyId}/shares`;
  const params = new URLSearchParams();
  if (options?.limit) params.append('limit', String(options.limit));
  if (options?.next_token) params.append('next_token', options.next_token);
  const qs = params.toString();
  if (qs) path += `?${qs}`;
  return client.get<ShareListResponse>(path);
}

/**
 * Update an existing share's permission scope
 */
export async function updateShare(
  client: ApiClient,
  shareId: string,
  request: UpdateShareRequest
): Promise<ApiResponse<ShareResponse>> {
  return client.put<ShareResponse>(`/shares/${shareId}`, request);
}

/**
 * Revoke (delete) a share
 */
export async function revokeShare(client: ApiClient, shareId: string): Promise<ApiResponse<void>> {
  return client.delete<void>(`/shares/${shareId}`);
}

/**
 * Accept a pending share invitation
 */
export async function acceptShare(
  client: ApiClient,
  shareId: string
): Promise<ApiResponse<ShareResponse>> {
  return client.post<ShareResponse>(`/shares/${shareId}/accept`, {});
}

/**
 * Get all shares for the authenticated user (accepter view) with optional pagination
 */
export async function getSharesForAccepter(
  client: ApiClient,
  options?: { limit?: number; next_token?: string }
): Promise<ApiResponse<ShareListResponse>> {
  let path = '/shares';
  const params = new URLSearchParams();
  if (options?.limit) params.append('limit', String(options.limit));
  if (options?.next_token) params.append('next_token', options.next_token);
  const qs = params.toString();
  if (qs) path += `?${qs}`;
  return client.get<ShareListResponse>(path);
}
