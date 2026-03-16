import { ApiClient } from './client';
import type { ApiResponse } from './client';
import type {
  CreateDependentRequest,
  UpdateDependentRequest,
  DependentResponse,
  DependentListResponse,
} from './types';

/**
 * Get all dependents for a specific family
 */
export async function getDependents(
  client: ApiClient,
  familyId: string
): Promise<ApiResponse<DependentListResponse>> {
  return client.get<DependentListResponse>(`/families/${familyId}/dependents`);
}

/**
 * Get a single dependent by ID
 */
export async function getDependent(
  client: ApiClient,
  familyId: string,
  dependentId: string
): Promise<ApiResponse<DependentResponse>> {
  return client.get<DependentResponse>(`/families/${familyId}/dependents/${dependentId}`);
}

/**
 * Create a new dependent
 */
export async function createDependent(
  client: ApiClient,
  familyId: string,
  request: CreateDependentRequest
): Promise<ApiResponse<DependentResponse>> {
  return client.post<DependentResponse>(`/families/${familyId}/dependents`, request);
}

/**
 * Update an existing dependent
 */
export async function updateDependent(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  request: UpdateDependentRequest
): Promise<ApiResponse<DependentResponse>> {
  return client.put<DependentResponse>(`/families/${familyId}/dependents/${dependentId}`, request);
}

/**
 * Delete a dependent
 */
export async function deleteDependent(
  client: ApiClient,
  familyId: string,
  dependentId: string
): Promise<ApiResponse<void>> {
  return client.delete<void>(`/families/${familyId}/dependents/${dependentId}`);
}
