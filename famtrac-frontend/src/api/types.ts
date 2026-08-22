// Family API types

export interface CreateFamilyRequest {
  name: string;
}

export interface UpdateFamilyRequest {
  name: string;
}

export interface FamilyResponse {
  id: string;
  name: string;
  owner_id: string;
  created_at: string;
  updated_at: string;
}

export interface FamilyListResponse {
  families: FamilyResponse[];
  next_token?: string;
}

// Dependent API types

export interface CreateDependentRequest {
  family_id: string;
  name: string;
  date_of_birth: string; // ISO 8601 format
}

export interface UpdateDependentRequest {
  name: string;
  date_of_birth: string;
}

export interface DependentResponse {
  id: string;
  family_id: string;
  name: string;
  date_of_birth: string;
  created_at: string;
  updated_at: string;
}

export interface DependentListResponse {
  dependents: DependentResponse[];
  next_token?: string;
}

// Activity API types

export type FeedingType = 'breast' | 'bottle' | 'solid';
export type DiaperContents = 'wet' | 'dirty' | 'both';
export type ActivityType =
  | 'feeding'
  | 'diaper_change'
  | 'sleep'
  | 'pumping'
  | 'activity_time'
  | 'tummy_time'
  | 'wake_window'
  | 'bath';

export interface CreateActivityRequest {
  family_id: string;
  dependent_id: string;
  type: ActivityType;
  timestamp: string;
  feeding_type?: FeedingType;
  contents?: DiaperContents;
  start_time?: string;
  end_time?: string;
  volume_ml?: number;
  medicine_added?: boolean;
  description?: string;
  notes?: string;
}

export interface UpdateActivityRequest {
  type: ActivityType;
  timestamp: string;
  feeding_type?: FeedingType;
  contents?: DiaperContents;
  start_time?: string;
  end_time?: string;
  volume_ml?: number;
  medicine_added?: boolean;
  description?: string;
  notes?: string;
}

export interface ActivityResponse {
  id: string;
  dependent_id: string;
  type: ActivityType;
  timestamp: string;
  created_at: string;
  updated_at: string;
  feeding_type?: FeedingType;
  contents?: DiaperContents;
  start_time?: string;
  end_time?: string;
  volume_ml?: number;
  medicine_added?: boolean;
  description?: string;
  notes?: string;
}

export interface ActivityListResponse {
  activities: ActivityResponse[];
  next_token?: string;
}

// Error response

export interface ErrorResponse {
  error: string;
  details?: string[];
}

// Share API types

export interface CreateShareRequest {
  accepter_username: string;
  permission_scope: { actions: string[] };
}

export interface UpdateShareRequest {
  permission_scope: { actions: string[] };
}

export interface ShareResponse {
  id: string;
  family_id: string;
  requester_id: string;
  accepter_username: string;
  accepter_id?: string;
  permission_scope: { actions: string[] };
  status: string;
  created_at: string;
  updated_at: string;
  expires_at?: string;
}

export interface ShareListResponse {
  shares: ShareResponse[];
  next_token?: string;
}

// Recipe API types

export interface CreateRecipeRequest {
  name: string;
  emoji?: string;
  ingredients?: string[];
  age_min?: number;
  texture?: string;
  allergens?: string[];
  prep_notes?: string;
  safe?: boolean;
}

export interface UpdateRecipeRequest {
  name?: string;
  emoji?: string;
  ingredients?: string[];
  age_min?: number;
  texture?: string;
  allergens?: string[];
  prep_notes?: string;
  safe?: boolean;
}

export interface RecipeResponse {
  id: string;
  family_id: string;
  name: string;
  emoji?: string;
  ingredients: string[];
  age_min?: number;
  texture?: string;
  allergens: string[];
  prep_notes?: string;
  safe?: boolean;
  created_at: string;
  updated_at: string;
  share_id?: string;
  permission_scope?: { actions: string[] };
}

export interface RecipeListResponse {
  recipes: RecipeResponse[];
  next_token?: string;
}

// MealSlot API types

export interface CreateMealSlotRequest {
  family_id: string;
  dependent_id: string;
  day: string;
  time: string;
  recipe_id?: string;
  notes?: string;
}

export interface UpdateMealSlotRequest {
  day?: string;
  time?: string;
  recipe_id?: string;
  notes?: string;
}

export interface MealSlotResponse {
  id: string;
  family_id: string;
  dependent_id: string;
  day: string;
  time: string;
  recipe_id?: string;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface MealSlotListResponse {
  meal_slots: MealSlotResponse[];
  next_token?: string;
}
