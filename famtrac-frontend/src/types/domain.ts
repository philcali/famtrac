// Domain model types for the famtrac-frontend application

export interface Family {
  id: string;
  name: string;
  owner_id: string;
  created_at: string;
  updated_at: string;
}

export interface Dependent {
  id: string;
  family_id: string;
  name: string;
  date_of_birth: string;
  created_at: string;
  updated_at: string;
}

// Activity type enums
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

// Base activity interface
export interface BaseActivity {
  id: string;
  dependent_id: string;
  activity_type: ActivityType;
  timestamp: string;
  created_at: string;
  updated_at: string;
}

// Specific activity types
export interface FeedingActivity extends BaseActivity {
  activity_type: 'feeding';
  feeding_type: FeedingType;
  medicine_added?: boolean;
}

export interface DiaperActivity extends BaseActivity {
  activity_type: 'diaper_change';
  contents: DiaperContents;
}

export interface SleepActivity extends BaseActivity {
  activity_type: 'sleep';
  start_time: string;
  end_time?: string;
}

export interface PumpingActivity extends BaseActivity {
  activity_type: 'pumping';
  volume_ml: number;
}

export interface ActivityTimeActivity extends BaseActivity {
  activity_type: 'activity_time';
  start_time: string;
  end_time?: string;
  description?: string;
}

export interface TummyTimeActivity extends BaseActivity {
  activity_type: 'tummy_time';
  start_time: string;
  end_time?: string;
  notes?: string;
}

export interface WakeWindowActivity extends BaseActivity {
  activity_type: 'wake_window';
  start_time: string;
  end_time?: string;
}

export interface BathActivity extends BaseActivity {
  activity_type: 'bath';
  start_time: string;
  end_time?: string;
  notes?: string;
}

// Union type for all activities
export type Activity =
  | FeedingActivity
  | DiaperActivity
  | SleepActivity
  | PumpingActivity
  | ActivityTimeActivity
  | TummyTimeActivity
  | WakeWindowActivity
  | BathActivity;

// Share types

export type PermissionAction =
  | 'family_read'
  | 'dependent_read'
  | 'dependent_write'
  | 'activity_read'
  | 'activity_write';

export interface PermissionScope {
  actions: PermissionAction[];
}

export type ShareStatus = 'pending' | 'active' | 'expired';

export interface Share {
  id: string;
  family_id: string;
  requester_id: string;
  accepter_username: string;
  accepter_id?: string;
  permission_scope: PermissionScope;
  status: ShareStatus;
  created_at: string;
  updated_at: string;
  expires_at?: string;
}

// Recipe types

export interface Recipe {
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
  permission_scope?: PermissionScope;
}

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

export interface PaginatedResponse<T> {
  items: T[];
  next_token?: string;
  total_count?: number;
}

// MealSlot domain types

export interface MealSlot {
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
