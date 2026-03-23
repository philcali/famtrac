# Requirements Document

## Introduction

The Activity Reporting feature adds a reporting view for a dependent that summarizes their activities over a configurable time range and visualizes the data using charts. Users can select from quick-access preset ranges (Today, This Week, This Month) or specify a custom date range. Feeding and pumping data are displayed as line graphs showing volume over time, and sleep data is displayed as a bar graph showing duration per day. Diaper change data is displayed as a bar graph showing count per day.

## Glossary

- **Report_Page**: The page component that displays activity summaries and charts for a single dependent within a given time range.
- **Time_Range_Selector**: The UI control that allows users to pick a date range, including preset quick links and custom date inputs.
- **Activity_Summary**: A section that displays aggregated statistics for each activity type within the selected time range.
- **Activity_Chart**: A chart component that visualizes activity data over time using line or bar graphs.
- **Dependent**: A child or person being tracked within a family.
- **Activity**: A recorded event for a dependent (feeding, diaper_change, sleep, or pumping).

## Requirements

### Requirement 1: Report Page Navigation

**User Story:** As a caregiver, I want to navigate to a reporting page for a dependent, so that I can view summarized activity data.

#### Acceptance Criteria

1. WHEN the caregiver clicks a "Reports" link on the DependentDetailPage, THE Report_Page SHALL navigate to the reporting view for that dependent.
2. THE Report_Page SHALL display the dependent's name in the page heading.
3. THE Report_Page SHALL provide a navigation link back to the DependentDetailPage.
4. THE Report_Page SHALL be accessible at the route `/families/:familyId/dependents/:dependentId/reports`.

### Requirement 2: Time Range Selection

**User Story:** As a caregiver, I want to select a time range for the report, so that I can view activity data for a specific period.

#### Acceptance Criteria

1. THE Time_Range_Selector SHALL display three preset quick-link buttons: "Today", "This Week", and "This Month".
2. WHEN the caregiver clicks "Today", THE Time_Range_Selector SHALL set the date range to the current calendar day (midnight to end of day).
3. WHEN the caregiver clicks "This Week", THE Time_Range_Selector SHALL set the date range from the most recent Monday to the current day.
4. WHEN the caregiver clicks "This Month", THE Time_Range_Selector SHALL set the date range from the first day of the current month to the current day.
5. THE Time_Range_Selector SHALL display custom start date and end date input fields.
6. WHEN the caregiver enters a custom start date and end date, THE Time_Range_Selector SHALL use those dates as the report range.
7. THE Time_Range_Selector SHALL visually highlight the currently active preset button.
8. WHEN the caregiver selects a custom date range, THE Time_Range_Selector SHALL deselect any active preset button.
9. THE Time_Range_Selector SHALL default to the "Today" preset on initial page load.

### Requirement 3: Activity Data Fetching

**User Story:** As a caregiver, I want the report to load all activities for the selected time range, so that the summaries and charts are accurate.

#### Acceptance Criteria

1. WHEN the time range changes, THE Report_Page SHALL fetch activities from the API using the selected start date and end date parameters.
2. WHILE activities are loading, THE Report_Page SHALL display a loading indicator.
3. IF the API returns an error, THEN THE Report_Page SHALL display an error message describing the failure.
4. THE Report_Page SHALL fetch all pages of activities by following pagination tokens until all data for the time range is retrieved.

### Requirement 4: Feeding Activity Summary and Chart

**User Story:** As a caregiver, I want to see a summary and line graph of feeding activities, so that I can track feeding patterns and volume over time.

#### Acceptance Criteria

1. THE Activity_Summary SHALL display the total number of feeding activities within the selected time range.
2. THE Activity_Summary SHALL display the total feeding volume in milliliters for feedings that have volume data.
3. THE Activity_Summary SHALL display the average feeding volume in milliliters per feeding (for feedings with volume data).
4. THE Activity_Chart SHALL render a line graph plotting feeding volume in milliliters on the Y-axis against time on the X-axis.
5. WHEN no feeding activities exist in the selected time range, THE Activity_Summary SHALL display zero for all feeding metrics.
6. WHEN no feeding activities exist in the selected time range, THE Activity_Chart SHALL display an empty state message instead of an empty graph.

### Requirement 5: Sleep Activity Summary and Chart

**User Story:** As a caregiver, I want to see a summary and bar graph of sleep activities, so that I can track sleep patterns and duration over time.

#### Acceptance Criteria

1. THE Activity_Summary SHALL display the total number of sleep activities within the selected time range.
2. THE Activity_Summary SHALL display the total sleep duration in hours and minutes.
3. THE Activity_Summary SHALL display the average sleep duration per sleep session in hours and minutes.
4. THE Activity_Chart SHALL render a bar graph plotting total sleep duration in minutes per calendar day on the Y-axis against each day on the X-axis.
5. WHEN no sleep activities exist in the selected time range, THE Activity_Summary SHALL display zero for all sleep metrics.
6. WHEN no sleep activities exist in the selected time range, THE Activity_Chart SHALL display an empty state message instead of an empty graph.

### Requirement 6: Diaper Change Activity Summary and Chart

**User Story:** As a caregiver, I want to see a summary and chart of diaper change activities, so that I can monitor diaper change frequency.

#### Acceptance Criteria

1. THE Activity_Summary SHALL display the total number of diaper change activities within the selected time range.
2. THE Activity_Summary SHALL display the count of wet, dirty, and both diaper changes separately.
3. THE Activity_Chart SHALL render a bar graph plotting the number of diaper changes per calendar day on the Y-axis against each day on the X-axis.
4. WHEN no diaper change activities exist in the selected time range, THE Activity_Summary SHALL display zero for all diaper change metrics.
5. WHEN no diaper change activities exist in the selected time range, THE Activity_Chart SHALL display an empty state message instead of an empty graph.

### Requirement 7: Pumping Activity Summary and Chart

**User Story:** As a caregiver, I want to see a summary and line graph of pumping activities, so that I can track pumping volume over time.

#### Acceptance Criteria

1. THE Activity_Summary SHALL display the total number of pumping activities within the selected time range.
2. THE Activity_Summary SHALL display the total pumping volume in milliliters.
3. THE Activity_Summary SHALL display the average pumping volume in milliliters per session.
4. THE Activity_Chart SHALL render a line graph plotting pumping volume in milliliters on the Y-axis against time on the X-axis.
5. WHEN no pumping activities exist in the selected time range, THE Activity_Summary SHALL display zero for all pumping metrics.
6. WHEN no pumping activities exist in the selected time range, THE Activity_Chart SHALL display an empty state message instead of an empty graph.

### Requirement 8: Chart Library Integration

**User Story:** As a developer, I want to use a lightweight charting library, so that charts render correctly and the bundle size stays reasonable.

#### Acceptance Criteria

1. THE Activity_Chart SHALL use the Recharts library for rendering line and bar graphs.
2. THE Activity_Chart SHALL render responsive charts that adapt to the container width.
3. THE Activity_Chart SHALL display axis labels for both X-axis (time/date) and Y-axis (value/count).
4. THE Activity_Chart SHALL display a tooltip showing the exact value when the caregiver hovers over a data point.

### Requirement 9: Empty State and Edge Cases

**User Story:** As a caregiver, I want clear feedback when there is no data, so that I understand the report is working but has nothing to show.

#### Acceptance Criteria

1. WHEN no activities of any type exist in the selected time range, THE Report_Page SHALL display a message indicating no activities were found for the selected period.
2. IF a sleep activity has a missing start_time or end_time, THEN THE Report_Page SHALL exclude that activity from the sleep duration calculations.
3. IF a feeding or pumping activity has no volume_ml value, THEN THE Report_Page SHALL exclude that activity from volume-based calculations.
