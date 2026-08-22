import { ApiClient } from './client';
import type { ApiResponse } from './client';
import type {
  CreateRecipeRequest,
  UpdateRecipeRequest,
  RecipeResponse,
  RecipeListResponse,
} from './types';

/**
 * Get all recipes for a specific family
 */
export async function getRecipes(
  client: ApiClient,
  familyId: string,
  limit?: number,
  nextToken?: string
): Promise<ApiResponse<RecipeListResponse>> {
  const params = new URLSearchParams();
  if (limit) params.append('limit', String(limit));
  if (nextToken) params.append('next_token', nextToken);
  const query = params.toString();
  const path = query ? `/families/${familyId}/recipes?${query}` : `/families/${familyId}/recipes`;
  return client.get<RecipeListResponse>(path);
}

/**
 * Get a single recipe by ID
 */
export async function getRecipe(
  client: ApiClient,
  familyId: string,
  recipeId: string
): Promise<ApiResponse<RecipeResponse>> {
  return client.get<RecipeResponse>(`/families/${familyId}/recipes/${recipeId}`);
}

/**
 * Create a new recipe
 */
export async function createRecipe(
  client: ApiClient,
  familyId: string,
  request: CreateRecipeRequest
): Promise<ApiResponse<RecipeResponse>> {
  return client.post<RecipeResponse>(`/families/${familyId}/recipes`, request);
}

/**
 * Update an existing recipe
 */
export async function updateRecipe(
  client: ApiClient,
  familyId: string,
  recipeId: string,
  request: UpdateRecipeRequest
): Promise<ApiResponse<RecipeResponse>> {
  return client.put<RecipeResponse>(`/families/${familyId}/recipes/${recipeId}`, request);
}

/**
 * Delete a recipe
 */
export async function deleteRecipe(
  client: ApiClient,
  familyId: string,
  recipeId: string
): Promise<ApiResponse<void>> {
  return client.delete<void>(`/families/${familyId}/recipes/${recipeId}`);
}
