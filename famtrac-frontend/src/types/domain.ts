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
export type ActivityType = 'feeding' | 'diaper_change' | 'sleep' | 'pumping';

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
}

export interface DiaperActivity extends BaseActivity {
  activity_type: 'diaper_change';
  contents: DiaperContents;
}

export interface SleepActivity extends BaseActivity {
  activity_type: 'sleep';
  start_time: string;
  end_time: string;
}

export interface PumpingActivity extends BaseActivity {
  activity_type: 'pumping';
  volume_ml: number;
}

// Union type for all activities
export type Activity = FeedingActivity | DiaperActivity | SleepActivity | PumpingActivity;

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
