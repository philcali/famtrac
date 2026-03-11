import { ApiClient } from './client';
import type { ApiResponse } from './client';
import type {
  CreateFamilyRequest,
  UpdateFamilyRequest,
  FamilyResponse,
  FamilyListResponse,
} from './types';

/**
 * Get all families for the authenticated user
 */
export async function getFamilies(client: ApiClient): Promise<ApiResponse<FamilyListResponse>> {
  return client.get<FamilyListResponse>('/families');
}

/**
 * Get a single family by ID
 */
export async function getFamily(
  client: ApiClient,
  familyId: string
): Promise<ApiResponse<FamilyResponse>> {
  return client.get<FamilyResponse>(`/families/${familyId}`);
}

/**
 * Create a new family
 */
export async function createFamily(
  client: ApiClient,
  request: CreateFamilyRequest
): Promise<ApiResponse<FamilyResponse>> {
  return client.post<FamilyResponse>('/families', request);
}

/**
 * Update an existing family
 */
export async function updateFamily(
  client: ApiClient,
  familyId: string,
  request: UpdateFamilyRequest
): Promise<ApiResponse<FamilyResponse>> {
  return client.put<FamilyResponse>(`/families/${familyId}`, request);
}

/**
 * Delete a family
 */
export async function deleteFamily(
  client: ApiClient,
  familyId: string
): Promise<ApiResponse<void>> {
  return client.delete<void>(`/families/${familyId}`);
}
