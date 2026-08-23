import { ApiClient } from './client';
import type { ApiResponse } from './client';
import type {
  CreateMealSlotRequest,
  UpdateMealSlotRequest,
  MealSlotResponse,
  MealSlotListResponse,
} from './types';

/**
 * Get all meal slots for a specific dependent
 */
export async function getMealSlots(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  day?: string,
  limit?: number,
  nextToken?: string
): Promise<ApiResponse<MealSlotListResponse>> {
  const params = new URLSearchParams();
  if (day) params.append('day', day);
  if (limit) params.append('limit', String(limit));
  if (nextToken) params.append('next_token', nextToken);
  const query = params.toString();
  const path = query
    ? `/families/${familyId}/dependents/${dependentId}/meal-slots?${query}`
    : `/families/${familyId}/dependents/${dependentId}/meal-slots`;
  return client.get<MealSlotListResponse>(path);
}

/**
 * Get a single meal slot by ID
 */
export async function getMealSlot(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  mealSlotId: string
): Promise<ApiResponse<MealSlotResponse>> {
  return client.get<MealSlotResponse>(
    `/families/${familyId}/dependents/${dependentId}/meal-slots/${mealSlotId}`
  );
}

/**
 * Create a new meal slot
 */
export async function createMealSlot(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  request: CreateMealSlotRequest
): Promise<ApiResponse<MealSlotResponse>> {
  return client.post<MealSlotResponse>(
    `/families/${familyId}/dependents/${dependentId}/meal-slots`,
    request
  );
}

/**
 * Update an existing meal slot
 */
export async function updateMealSlot(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  mealSlotId: string,
  request: UpdateMealSlotRequest
): Promise<ApiResponse<MealSlotResponse>> {
  return client.put<MealSlotResponse>(
    `/families/${familyId}/dependents/${dependentId}/meal-slots/${mealSlotId}`,
    request
  );
}

/**
 * Delete a meal slot
 */
export async function deleteMealSlot(
  client: ApiClient,
  familyId: string,
  dependentId: string,
  mealSlotId: string
): Promise<ApiResponse<void>> {
  return client.delete<void>(
    `/families/${familyId}/dependents/${dependentId}/meal-slots/${mealSlotId}`
  );
}
