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

// Activity API types

export type FeedingType = 'breast' | 'bottle' | 'solid';
export type DiaperContents = 'wet' | 'dirty' | 'both';
export type ActivityType = 'feeding' | 'diaper_change' | 'sleep' | 'pumping';

export interface CreateActivityRequest {
  dependent_id: string;
  activity_type: ActivityType;
  timestamp: string;
  feeding_type?: FeedingType;
  contents?: DiaperContents;
  start_time?: string;
  end_time?: string;
  volume_ml?: number;
}

export interface UpdateActivityRequest {
  timestamp: string;
  feeding_type?: FeedingType;
  contents?: DiaperContents;
  start_time?: string;
  end_time?: string;
  volume_ml?: number;
}

export interface ActivityResponse {
  id: string;
  dependent_id: string;
  activity_type: ActivityType;
  timestamp: string;
  created_at: string;
  updated_at: string;
  feeding_type?: FeedingType;
  contents?: DiaperContents;
  start_time?: string;
  end_time?: string;
  volume_ml?: number;
}

export interface ActivityListResponse {
  activities: ActivityResponse[];
}

// Error response

export interface ErrorResponse {
  error: string;
  details?: string[];
}
