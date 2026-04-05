# Implementation Plan: Bootstrap Icons Integration

## Overview

This plan implements the Bootstrap Icons integration feature across the Rust backend and React/TypeScript frontend. It starts with backend enum extensions, then builds the frontend icon infrastructure, updates existing components, adds new activity type support, and finishes with report page enhancements. Each task builds incrementally on the previous ones.

## Tasks

- [x] 1. Add new activity type variants to the backend
  - [x] 1.1 Add `ActivityTime`, `TummyTime`, and `WakeWindow` variants to the `ActivityType` enum in `famtrac-backend/src/domain/activity.rs`
    - Add `ActivityTime` variant with `start_time: Timestamp`, `end_time: Option<Timestamp>`, `description: Option<String>` fields
    - Add `TummyTime` variant with `start_time: Timestamp`, `end_time: Option<Timestamp>`, `notes: Option<String>` fields
    - Add `WakeWindow` variant with `start_time: Timestamp`, `end_time: Option<Timestamp>` fields
    - All optional fields must use `#[serde(skip_serializing_if = "Option::is_none")]`
    - The existing `#[serde(tag = "type", rename_all = "snake_case")]` attribute handles snake_case serialization
    - _Requirements: 7.1, 7.2, 7.3, 7.7_

  - [x] 1.2 Add validation arms for the new variants in `famtrac-backend/src/errors/validation.rs`
    - Add `ActivityTime` arm to `validate_activity_type`: validate `end_time > start_time` if `end_time` is `Some`; validate `description` length ≤ 500 if present using `sanitize_string`
    - Add `TummyTime` arm: validate `end_time > start_time` if `end_time` is `Some`; validate `notes` length ≤ 500 if present using `sanitize_string`
    - Add `WakeWindow` arm: validate `end_time > start_time` if `end_time` is `Some`
    - _Requirements: 7.4, 7.5, 7.6_

  - [ ]* 1.3 Write property test for backend ActivityType serialization round-trip
    - **Property 5: Backend ActivityType serialization round-trip**
    - Generate arbitrary `ActivityType` values for all 7 variants using `proptest` strategies
    - Serialize each to JSON with `serde_json::to_string`, deserialize back with `serde_json::from_str`, assert equality
    - Assert the `"type"` field in the serialized JSON matches the expected snake_case string (`"activity_time"`, `"tummy_time"`, `"wake_window"`)
    - Add to `famtrac-backend/tests/domain_serialization_test.rs`
    - **Validates: Requirements 7.1, 7.2, 7.3, 7.7**

- [ ] 2. Checkpoint - Ensure backend compiles and tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 3. Create frontend icon infrastructure
  - [x] 3.1 Create the icon registry utility at `famtrac-frontend/src/utils/iconRegistry.ts`
    - Export a `iconRegistry` record mapping icon name strings to Bootstrap Icons SVG markup strings
    - Include entries for: `pencil`, `trash`, `eye`, `plus`, `stop` (bi-stop-fill), `feeding` (e.g. bi-cup-straw), `diaper_change` (e.g. bi-droplet), `sleep` (e.g. bi-moon), `pumping` (e.g. bi-moisture), `activity_time` (e.g. bi-controller), `tummy_time` (e.g. bi-person-arms-up), `wake_window` (e.g. bi-sun)
    - Export a `getIcon(name: string): string | undefined` function that looks up the registry
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 8.1, 9.1, 10.1_

  - [ ]* 3.2 Write property test for icon registry lookup correctness
    - **Property 1: Icon registry lookup correctness**
    - Use `fast-check` with `fc.string()` to generate arbitrary strings
    - If the string is in the known icon set, assert `getIcon(name)` returns a non-empty string
    - If the string is not in the known icon set, assert `getIcon(name)` returns `undefined`
    - Create test file at `famtrac-frontend/src/utils/iconRegistry.test.ts`
    - **Validates: Requirements 1.3, 1.4, 1.5**

  - [x] 3.3 Create the Icon React component at `famtrac-frontend/src/components/common/Icon.tsx`
    - Accept `name: string`, optional `size: number` (default 16), optional `className: string` props
    - Look up `name` in the icon registry via `getIcon`
    - Return `null` if not found
    - Render SVG via `dangerouslySetInnerHTML` on a `<span>` wrapper with `aria-hidden="true"`, `width` and `height` set to `size`
    - Apply `className` to the wrapper span
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

  - [ ]* 3.4 Write property test for Icon component rendering correctness
    - **Property 2: Icon component rendering correctness**
    - Use `fast-check` to generate tuples of (arbitrary string, positive integer size, optional className)
    - Render `<Icon>` with React Testing Library
    - If name is valid: assert rendered element has `aria-hidden="true"`, correct width/height, and className applied
    - If name is invalid: assert nothing is rendered
    - Create test file at `famtrac-frontend/src/components/common/Icon.test.tsx`
    - **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**

- [x] 4. Create formatDuration utility and integrate into existing components
  - [x] 4.1 Create `famtrac-frontend/src/utils/formatDuration.ts`
    - Export `formatDuration(totalMinutes: number): string` that returns `"Xh Ym"` format
    - `formatDuration(150)` → `"2h 30m"`, `formatDuration(45)` → `"0h 45m"`, `formatDuration(0)` → `"0h 0m"`
    - _Requirements: 5.1, 5.3_

  - [ ]* 4.2 Write property test for duration formatting
    - **Property 4: Duration formatting**
    - Use `fast-check` with `fc.nat()` to generate non-negative integers
    - Call `formatDuration(n)` and assert output matches `"Xh Ym"` where `X = floor(n/60)` and `Y = n % 60`
    - Assert parsing `X * 60 + Y` back equals the original input
    - Create test file at `famtrac-frontend/src/utils/formatDuration.test.ts`
    - **Validates: Requirements 5.1, 5.3, 8.5, 9.5, 10.5**

  - [x] 4.3 Update `famtrac-frontend/src/components/activities/formats.tsx` to use `formatDuration`
    - Import `formatDuration` from `../../utils/formatDuration`
    - In `renderActivityDetails` sleep case: replace the minutes calculation with `formatDuration(minutes)` and show "In Progress" when `end_time` is absent
    - Add `getActivityTypeLabel` cases: `activity_time` → "Activity Time", `tummy_time` → "Tummy Time", `wake_window` → "Wake Window"
    - Add `getActivityTypeBadgeVariant` cases for the three new types
    - Add `renderActivityDetails` cases for `activity_time`, `tummy_time`, `wake_window` using `formatDuration` for duration and "In Progress" when `end_time` is absent
    - _Requirements: 5.1, 8.5, 8.6, 9.5, 9.6, 10.5, 10.6_

- [x] 5. Update frontend type definitions for new activity types
  - [x] 5.1 Update `famtrac-frontend/src/types/domain.ts`
    - Extend `ActivityType` union to include `'activity_time' | 'tummy_time' | 'wake_window'`
    - Make `SleepActivity.end_time` optional (`end_time?: string`) to support stopwatch mode
    - Add `ActivityTimeActivity` interface extending `BaseActivity` with `activity_type: 'activity_time'`, `start_time: string`, `end_time?: string`, `description?: string`
    - Add `TummyTimeActivity` interface extending `BaseActivity` with `activity_type: 'tummy_time'`, `start_time: string`, `end_time?: string`, `notes?: string`
    - Add `WakeWindowActivity` interface extending `BaseActivity` with `activity_type: 'wake_window'`, `start_time: string`, `end_time?: string`
    - Add the three new interfaces to the `Activity` union type
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

  - [x] 5.2 Update `famtrac-frontend/src/api/types.ts`
    - Extend `ActivityType` to include `'activity_time' | 'tummy_time' | 'wake_window'`
    - Add `description?: string` and `notes?: string` optional fields to `CreateActivityRequest`, `UpdateActivityRequest`, and `ActivityResponse`
    - _Requirements: 11.6_

- [-] 6. Integrate icons into action buttons and activity cards
  - [x] 6.1 Add optional `icon` prop to `famtrac-frontend/src/components/common/Button.tsx`
    - Add `icon?: string` to `ButtonProps`
    - When `icon` is provided, render `<Icon name={icon} size={14} className="me-1" />` before `children`
    - Existing text labels must be preserved
    - Export `Icon` from `famtrac-frontend/src/components/common/index.ts`
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

  - [ ]* 6.2 Write property test for button text preservation with icon
    - **Property 3: Button text preservation with icon**
    - Use `fast-check` to generate (valid icon name, text string) pairs
    - Render `<Button icon={name}>{text}</Button>` and assert the text is present in the output
    - Add to `famtrac-frontend/src/components/common/Button.test.tsx`
    - **Validates: Requirements 3.5**

  - [x] 6.3 Update `famtrac-frontend/src/components/activities/ActivityCard.tsx` to show activity type icons and stop icon
    - Import `Icon` component
    - Render `<Icon name={activity.type} size={14} className="me-1" />` inside the `<Badge>` next to the type label
    - For stopwatch-type activities (sleep, activity_time, tummy_time, wake_window) without `end_time`, render `<Icon name="stop" size={14} className="text-danger ms-1" />` next to "In Progress" text
    - Do not render the stop icon when `end_time` is present
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 6.6, 6.7, 6.8, 8.4, 8.6, 8.8, 8.9, 9.4, 9.6, 9.8, 9.9, 10.4, 10.6, 10.8, 10.9_

  - [ ]* 6.4 Write property test for stop icon visibility
    - **Property 6: Stop icon visibility for in-progress stopwatch activities**
    - Use `fast-check` to generate arbitrary stopwatch-type activities (sleep, activity_time, tummy_time, wake_window) with and without `end_time`
    - Render `<ActivityCard>` and assert the stop icon is present if and only if `end_time` is absent
    - Create test file at `famtrac-frontend/src/components/activities/ActivityCard.test.tsx`
    - **Validates: Requirements 6.7, 6.8, 8.8, 8.9, 9.8, 9.9, 10.8, 10.9**

- [x] 7. Update action buttons throughout the app to use icons
  - [x] 7.1 Add icon props to existing action buttons across the application
    - Update Edit buttons to use `icon="pencil"`
    - Update Delete/Revoke buttons to use `icon="trash"`
    - Update View buttons to use `icon="eye"`
    - Update Add/Create/Invite buttons to use `icon="plus"`
    - Files to update: `ActivityCard.tsx`, `FamilyCard.tsx`, `DependentCard.tsx`, `ShareCard.tsx`, and any page-level action buttons in `FamiliesPage.tsx`, `FamilyDetailPage.tsx`, `DependentDetailPage.tsx`, `PendingSharesPage.tsx`
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 8. Implement stopwatch mode in ActivityForm
  - [x] 8.1 Update `famtrac-frontend/src/components/activities/ActivityForm.tsx` with stopwatch mode and new activity types
    - Add `activity_time`, `tummy_time`, `wake_window` options to the `<Form.Select>` activity type selector
    - Add a `stopwatchMode` boolean state, defaulting to `false`
    - For activity types `sleep`, `activity_time`, `tummy_time`, `wake_window`: show a "Stopwatch Mode" `<Form.Check type="switch">` toggle
    - When stopwatch mode is enabled: hide the end time field, use start_time as the activity timestamp, do not require end_time for validation
    - When stopwatch mode is disabled: require both start_time and end_time as before
    - For `activity_time`: show optional description text field
    - For `tummy_time`: show optional notes text field
    - For `wake_window`: show start_time and optional end_time only
    - Update the `onSubmit` data construction to include `description` and `notes` fields for the new types
    - Update validation rules to handle the new activity types (start_time required, end_time > start_time when both present)
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 8.2, 8.3, 8.7, 9.2, 9.3, 9.7, 10.2, 10.3, 10.7_

- [ ] 9. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. Update report page for hours display and wake window support
  - [x] 10.1 Update `famtrac-frontend/src/utils/reportUtils.ts` for hours display and wake window
    - Update `transformSleepChartData` to convert values from minutes to decimal hours (value / 60)
    - Add `computeWakeWindowSummary` function mirroring `computeSleepSummary` but filtering for `wake_window` type
    - Add `transformWakeWindowChartData` function mirroring `transformSleepChartData` but filtering for `wake_window` type
    - _Requirements: 5.2, 10.10_

  - [x] 10.2 Update `famtrac-frontend/src/pages/ReportPage.tsx` for hours display and wake window
    - Change sleep chart `yAxisLabel` from `"Duration (min)"` to `"Duration (hours)"`
    - Replace the local `formatMinutes` function with imported `formatDuration` from `../../utils/formatDuration`
    - Use `formatDuration` for sleep summary card total and average duration display
    - Add wake window summary card using `computeWakeWindowSummary`
    - Add wake window chart using `transformWakeWindowChartData`
    - _Requirements: 5.2, 5.3, 10.10_

- [ ] 11. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- Unit tests validate specific examples and edge cases
- The backend requires no handler changes — existing CRUD handlers use `ActivityType` generically via serde
- No DynamoDB schema changes are needed — activity_type is stored as a JSON-serialized attribute
