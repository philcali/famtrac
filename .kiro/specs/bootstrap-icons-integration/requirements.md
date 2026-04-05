# Requirements Document

## Introduction

This feature introduces a centralized Bootstrap Icons utility to the FamTrac frontend application. The utility provides a curated set of SVG icons accessible by name, which are then integrated into action buttons, activity type indicators, and navigation elements throughout the app. Additionally, this feature includes two small UI improvements: displaying sleep duration in hours instead of minutes, and adding a stopwatch-style toggle for tracking sleep activities without a known end time. The feature also introduces three new activity types — Activity Time, Tummy Time, and Wake Window — with full backend and frontend support including new API activity type variants, activity cards, forms, and icons in the icon registry.

## Glossary

- **Icon_Registry**: A TypeScript module that maps icon names to their SVG markup strings, providing a single source of truth for all icons used in the application
- **Icon_Component**: A React component that renders an SVG icon from the Icon_Registry given an icon name, with configurable size and color
- **Action_Button**: A button in the UI that performs a CRUD operation (e.g., Edit, Delete, View, Add, Accept, Revoke)
- **Activity_Card**: A card component that displays a single activity entry with its type, details, and action buttons
- **Activity_Form**: A form component used to create or edit activity entries
- **Report_Page**: The page that displays activity summaries and charts for a dependent over a configurable time range
- **Sleep_Activity**: An activity of type "sleep" that tracks a sleep session with start and end times
- **Stopwatch_Mode**: A sleep tracking mode where only a start time is recorded and no end time is set, indicating an ongoing sleep session
- **Activity_Time_Activity**: An activity of type "activity_time" that tracks a period of baby play or interactive engagement, recorded with a start time, optional end time, and optional description
- **Tummy_Time_Activity**: An activity of type "tummy_time" that tracks a supervised prone (face-down) exercise session for neck tone and muscle development, recorded with a start time, optional end time, and optional notes
- **Wake_Window_Activity**: An activity of type "wake_window" that tracks a continuous awake period between sleep sessions, recorded with a start time and optional end time, functioning similarly to Sleep_Activity but for awake periods
- **Backend_ActivityType_Enum**: The Rust enum `ActivityType` in `famtrac-backend/src/domain/activity.rs` that defines all supported activity type variants and their associated data

## Requirements

### Requirement 1: Icon Registry Utility

**User Story:** As a developer, I want a centralized icon registry that maps icon names to SVG strings, so that icons are managed in one place and can be reused consistently across the application.

#### Acceptance Criteria

1. THE Icon_Registry SHALL export a mapping of icon name strings to SVG markup strings for the following icons: pencil, trash, eye, plus, stop, and one icon per activity type (feeding, diaper change, sleep, pumping)
2. THE Icon_Registry SHALL use Bootstrap Icons SVG paths as the source for each icon
3. WHEN a valid icon name is provided, THE Icon_Registry SHALL return the corresponding SVG markup string
4. WHEN an invalid icon name is provided, THE Icon_Registry SHALL return undefined
5. THE Icon_Registry SHALL include a "stop" icon entry (e.g., bi-stop-fill) for indicating in-progress stopwatch activities

### Requirement 2: Icon React Component

**User Story:** As a developer, I want a reusable React component that renders icons from the registry, so that I can place icons in the UI with consistent sizing and accessibility.

#### Acceptance Criteria

1. THE Icon_Component SHALL accept an icon name prop and render the corresponding SVG from the Icon_Registry
2. THE Icon_Component SHALL accept an optional size prop that controls the width and height of the rendered SVG, defaulting to 16 pixels
3. THE Icon_Component SHALL accept an optional className prop for additional styling
4. THE Icon_Component SHALL set the aria-hidden attribute to true on the rendered SVG element
5. WHEN the icon name does not exist in the Icon_Registry, THE Icon_Component SHALL render nothing

### Requirement 3: Action Button Icons

**User Story:** As a user, I want to see recognizable icons on action buttons, so that I can quickly identify what each button does.

#### Acceptance Criteria

1. WHEN an Edit Action_Button is rendered, THE Action_Button SHALL display a pencil icon alongside the "Edit" text label
2. WHEN a Delete or Revoke Action_Button is rendered, THE Action_Button SHALL display a trash icon alongside the text label
3. WHEN a View Action_Button is rendered, THE Action_Button SHALL display an eye icon alongside the "View" text label
4. WHEN an Add, Create, or Invite Action_Button is rendered, THE Action_Button SHALL display a plus icon alongside the text label
5. THE Action_Button SHALL retain its existing text label when an icon is added

### Requirement 4: Activity Type Icons

**User Story:** As a user, I want to see distinct icons for each activity type on activity cards, so that I can visually distinguish between feeding, diaper change, sleep, and pumping activities at a glance.

#### Acceptance Criteria

1. WHEN an Activity_Card for a feeding activity is rendered, THE Activity_Card SHALL display a feeding icon next to the activity type badge
2. WHEN an Activity_Card for a diaper change activity is rendered, THE Activity_Card SHALL display a diaper change icon next to the activity type badge
3. WHEN an Activity_Card for a sleep activity is rendered, THE Activity_Card SHALL display a sleep icon next to the activity type badge
4. WHEN an Activity_Card for a pumping activity is rendered, THE Activity_Card SHALL display a pumping icon next to the activity type badge

### Requirement 5: Sleep Duration Display in Hours

**User Story:** As a user, I want sleep durations displayed in hours and minutes instead of only minutes, so that longer sleep sessions are easier to read.

#### Acceptance Criteria

1. WHEN a sleep activity duration is displayed on the Activity_Card, THE Activity_Card SHALL format the duration as hours and minutes (e.g., "2h 30m") instead of total minutes
2. WHEN the Report_Page sleep chart y-axis displays duration, THE Report_Page SHALL label the axis as "Duration (hours)" and display values in decimal hours
3. WHEN the Report_Page sleep summary card displays total or average duration, THE Report_Page SHALL format the value as hours and minutes (e.g., "2h 30m")

### Requirement 6: Sleep Stopwatch Mode

**User Story:** As a user, I want to start tracking a sleep session without specifying an end time, so that I can record when sleep begins and fill in the end time later.

#### Acceptance Criteria

1. WHEN the Activity_Form is set to sleep type, THE Activity_Form SHALL display a "Stopwatch Mode" toggle
2. WHEN Stopwatch_Mode is enabled, THE Activity_Form SHALL use the start time value as the activity timestamp
3. WHEN Stopwatch_Mode is enabled, THE Activity_Form SHALL hide the end time field
4. WHEN Stopwatch_Mode is enabled, THE Activity_Form SHALL not require an end time for form submission
5. WHEN Stopwatch_Mode is disabled, THE Activity_Form SHALL require both start time and end time as before
6. WHEN a Sleep_Activity without an end time is displayed on the Activity_Card, THE Activity_Card SHALL display "In Progress" instead of a duration value
7. WHEN a Sleep_Activity without an end_time is displayed on the Activity_Card, THE Activity_Card SHALL display a stop icon (square/bi-stop-fill) to indicate the activity is in progress
8. WHEN a Sleep_Activity has an end_time, THE Activity_Card SHALL NOT display the stop icon

### Requirement 7: New Activity Types — Backend Support

**User Story:** As a developer, I want the backend to support activity_time, tummy_time, and wake_window activity types, so that the API can store and serve these new activity records.

#### Acceptance Criteria

1. THE Backend_ActivityType_Enum SHALL include an `ActivityTime` variant with a `start_time` field of type Timestamp, an optional `end_time` field of type Timestamp, and an optional `description` field of type String
2. THE Backend_ActivityType_Enum SHALL include a `TummyTime` variant with a `start_time` field of type Timestamp, an optional `end_time` field of type Timestamp, and an optional `notes` field of type String
3. THE Backend_ActivityType_Enum SHALL include a `WakeWindow` variant with a `start_time` field of type Timestamp and an optional `end_time` field of type Timestamp
4. WHEN a create activity request with type "wake_window" is received, THE backend SHALL validate that the `start_time` field is present
5. WHEN a create activity request with type "activity_time" is received, THE backend SHALL validate that the `start_time` field is present
6. WHEN a create activity request with type "tummy_time" is received, THE backend SHALL validate that the `start_time` field is present
7. THE Backend_ActivityType_Enum SHALL serialize the new variants using snake_case naming consistent with existing variants (activity_time, tummy_time, wake_window)

### Requirement 8: Activity Time — Frontend Support

**User Story:** As a user, I want to log "Activity Time" entries for my baby, so that I can track play sessions and interactive engagement periods.

#### Acceptance Criteria

1. THE Icon_Registry SHALL include an icon entry for the activity_time activity type using a Bootstrap Icons SVG path
2. WHEN the Activity_Form type selector is rendered, THE Activity_Form SHALL include "Activity Time" as a selectable activity type
3. WHEN the Activity_Form is set to activity_time type, THE Activity_Form SHALL display a start time field, an optional end time field, and an optional description text field
4. WHEN an Activity_Card for an activity_time activity is rendered, THE Activity_Card SHALL display the activity_time icon next to the activity type badge
5. WHEN an activity_time activity has both start_time and end_time, THE Activity_Card SHALL display the duration formatted as hours and minutes (e.g., "1h 15m")
6. WHEN an activity_time activity has no end_time, THE Activity_Card SHALL display "In Progress" instead of a duration value
7. WHEN the Activity_Form is set to activity_time type, THE Activity_Form SHALL display a "Stopwatch Mode" toggle that behaves identically to the sleep Stopwatch_Mode
8. WHEN an activity_time activity has no end_time, THE Activity_Card SHALL display a stop icon (square/bi-stop-fill) to indicate the activity is in progress
9. WHEN an activity_time activity has an end_time, THE Activity_Card SHALL NOT display the stop icon

### Requirement 9: Tummy Time — Frontend Support

**User Story:** As a user, I want to log "Tummy Time" sessions for my baby, so that I can track supervised prone exercise for neck tone and muscle development.

#### Acceptance Criteria

1. THE Icon_Registry SHALL include an icon entry for the tummy_time activity type using a Bootstrap Icons SVG path
2. WHEN the Activity_Form type selector is rendered, THE Activity_Form SHALL include "Tummy Time" as a selectable activity type
3. WHEN the Activity_Form is set to tummy_time type, THE Activity_Form SHALL display a start time field, an optional end time field, and an optional notes text field
4. WHEN an Activity_Card for a tummy_time activity is rendered, THE Activity_Card SHALL display the tummy_time icon next to the activity type badge
5. WHEN a tummy_time activity has both start_time and end_time, THE Activity_Card SHALL display the duration formatted as hours and minutes (e.g., "0h 20m")
6. WHEN a tummy_time activity has no end_time, THE Activity_Card SHALL display "In Progress" instead of a duration value
7. WHEN the Activity_Form is set to tummy_time type, THE Activity_Form SHALL display a "Stopwatch Mode" toggle that behaves identically to the sleep Stopwatch_Mode
8. WHEN a tummy_time activity has no end_time, THE Activity_Card SHALL display a stop icon (square/bi-stop-fill) to indicate the activity is in progress
9. WHEN a tummy_time activity has an end_time, THE Activity_Card SHALL NOT display the stop icon

### Requirement 10: Wake Window — Frontend Support

**User Story:** As a user, I want to log "Wake Window" periods for my baby, so that I can track how long the baby stays awake between sleep sessions and optimize nap schedules.

#### Acceptance Criteria

1. THE Icon_Registry SHALL include an icon entry for the wake_window activity type using a Bootstrap Icons SVG path
2. WHEN the Activity_Form type selector is rendered, THE Activity_Form SHALL include "Wake Window" as a selectable activity type
3. WHEN the Activity_Form is set to wake_window type, THE Activity_Form SHALL display a start time field and an optional end time field
4. WHEN an Activity_Card for a wake_window activity is rendered, THE Activity_Card SHALL display the wake_window icon next to the activity type badge
5. WHEN a wake_window activity has both start_time and end_time, THE Activity_Card SHALL display the duration formatted as hours and minutes (e.g., "2h 45m")
6. WHEN a wake_window activity has no end_time, THE Activity_Card SHALL display "In Progress" instead of a duration value
7. WHEN the Activity_Form is set to wake_window type, THE Activity_Form SHALL display a "Stopwatch Mode" toggle that behaves identically to the sleep Stopwatch_Mode
8. WHEN a wake_window activity has no end_time, THE Activity_Card SHALL display a stop icon (square/bi-stop-fill) to indicate the activity is in progress
9. WHEN a wake_window activity has an end_time, THE Activity_Card SHALL NOT display the stop icon
10. WHEN the Report_Page displays wake_window activities, THE Report_Page SHALL include wake_window durations in summary statistics alongside sleep durations

### Requirement 11: Frontend Type Definitions for New Activity Types

**User Story:** As a developer, I want the frontend TypeScript type definitions to include the new activity types, so that type safety is maintained across the application.

#### Acceptance Criteria

1. THE ActivityType union type SHALL include "activity_time", "tummy_time", and "wake_window" as valid values
2. THE domain types SHALL define an ActivityTimeActivity interface with activity_type "activity_time", start_time string, optional end_time string, and optional description string
3. THE domain types SHALL define a TummyTimeActivity interface with activity_type "tummy_time", start_time string, optional end_time string, and optional notes string
4. THE domain types SHALL define a WakeWindowActivity interface with activity_type "wake_window", start_time string, and optional end_time string
5. THE Activity union type SHALL include ActivityTimeActivity, TummyTimeActivity, and WakeWindowActivity
6. THE API types SHALL include "activity_time", "tummy_time", and "wake_window" in the ActivityType type and include the corresponding optional fields in CreateActivityRequest, UpdateActivityRequest, and ActivityResponse
