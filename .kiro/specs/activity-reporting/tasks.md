# Implementation Plan: Activity Reporting

## Overview

Implement a reporting page for dependents that shows activity summaries and charts over configurable time ranges. Tasks are ordered for incremental usefulness: pure utility functions first, then the data hook, then summary UI wired end-to-end with routing, and finally charts as an enhancement layer.

## Tasks

- [x] 1. Create report utility functions and types
  - [x] 1.1 Create `src/utils/reportUtils.ts` with date range preset and summary computation functions
    - Define `TimeRangePreset`, `ChartDataPoint`, `FeedingSummary`, `SleepSummary`, `DiaperSummary`, `PumpingSummary` types
    - Implement `getPresetDateRange(preset)` returning `{ startDate, endDate }` ISO strings
    - Implement `computeFeedingSummary(activities)` — filter to feeding, count all, sum/avg volume only where `volume_ml` is defined
    - Implement `computeSleepSummary(activities)` — filter to sleep, exclude entries missing `start_time`/`end_time` from duration calcs
    - Implement `computeDiaperSummary(activities)` — filter to diaper_change, count by `contents` field
    - Implement `computePumpingSummary(activities)` — filter to pumping, sum/avg volume only where defined
    - Implement `transformFeedingChartData(activities)` — feeding activities with volume as `ChartDataPoint[]`
    - Implement `transformSleepChartData(activities)` — group valid sleep by calendar day, sum duration per day
    - Implement `transformDiaperChartData(activities)` — group diaper changes by calendar day, count per day
    - Implement `transformPumpingChartData(activities)` — pumping activities with volume as `ChartDataPoint[]`
    - _Requirements: 2.2, 2.3, 2.4, 4.1, 4.2, 4.3, 5.1, 5.2, 5.3, 6.1, 6.2, 7.1, 7.2, 7.3, 9.2, 9.3_

- [ ] 2. Property-based tests for pure utility functions
  - [ ]* 2.1 Write property test: Preset date range computation
    - **Property 1: Preset date range computation**
    - Generate random dates and presets, verify `getPresetDateRange` produces valid ranges with correct anchor points and `startDate <= endDate`
    - **Validates: Requirements 2.3, 2.4**

  - [ ]* 2.2 Write property test: Activity count by type
    - **Property 2: Activity count by type**
    - Generate random `ActivityResponse[]` with mixed types, verify each summary's `totalCount` matches a simple filter count
    - **Validates: Requirements 4.1, 5.1, 6.1, 7.1**

  - [ ]* 2.3 Write property test: Volume summary correctness for feeding and pumping
    - **Property 3: Volume summary correctness for feeding and pumping**
    - Generate random feeding/pumping activities with and without `volume_ml`, verify total and average calculations
    - **Validates: Requirements 4.2, 4.3, 7.2, 7.3, 9.3**

  - [ ]* 2.4 Write property test: Sleep duration summary correctness
    - **Property 4: Sleep duration summary correctness**
    - Generate random sleep activities with and without valid start/end times, verify duration calculations exclude invalid entries
    - **Validates: Requirements 5.2, 5.3, 9.2**

  - [ ]* 2.5 Write property test: Diaper sub-type counts partition the total
    - **Property 5: Diaper sub-type counts partition the total**
    - Generate random diaper change activities with random `contents` values, verify `wetCount + dirtyCount + bothCount === totalCount`
    - **Validates: Requirements 6.2**

  - [ ]* 2.6 Write property test: Sleep chart data groups durations by calendar day
    - **Property 6: Sleep chart data groups durations by calendar day**
    - Generate random valid sleep activities across multiple days, verify one point per day and sum matches total duration
    - **Validates: Requirements 5.4**

  - [ ]* 2.7 Write property test: Diaper chart data groups counts by calendar day
    - **Property 7: Diaper chart data groups counts by calendar day**
    - Generate random diaper activities across multiple days, verify one point per day and sum matches total count
    - **Validates: Requirements 6.3**

- [ ] 3. Checkpoint - Verify utility functions
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement `useReportData` hook for paginated data fetching
  - [x] 4.1 Create `src/hooks/useReportData.ts`
    - Implement pagination loop that follows `next_token` until all pages are fetched
    - Use `useCallback` + `useEffect` pattern matching existing `useApi` hook style
    - Expose `{ activities, loading, error }` state
    - Re-fetch when `startDate` or `endDate` changes
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

  - [ ]* 4.2 Write property test: Pagination accumulates all activities
    - **Property 8: Pagination accumulates all activities**
    - Generate random sequences of paginated responses, verify accumulated array length equals sum of page lengths
    - **Validates: Requirements 3.4**

- [x] 5. Implement TimeRangeSelector component
  - [x] 5.1 Create `src/components/reports/TimeRangeSelector.tsx`
    - Render three preset `ButtonGroup` buttons (Today, This Week, This Month) and two `Form.Control type="date"` inputs
    - Active preset uses `variant="primary"`, inactive use `variant="outline-primary"`
    - Call `onPresetSelect` on preset click, `onCustomRangeChange` on date input change
    - _Requirements: 2.1, 2.5, 2.6, 2.7, 2.8_

- [x] 6. Implement ActivitySummaryCard component
  - [x] 6.1 Create `src/components/reports/ActivitySummaryCard.tsx`
    - Render a Bootstrap Card with a colored header (`variant` prop), title, and list of label/value metric pairs
    - _Requirements: 4.1, 4.2, 4.3, 4.5, 5.1, 5.2, 5.3, 5.5, 6.1, 6.2, 6.4, 7.1, 7.2, 7.3, 7.5_

- [x] 7. Assemble ReportPage with route registration and navigation
  - [x] 7.1 Create `src/pages/ReportPage.tsx`
    - Read `familyId` and `dependentId` from route params
    - Fetch dependent details for page heading using `getDependent`
    - Manage `startDate`, `endDate`, `activePreset` state, defaulting to "Today" preset
    - Use `useReportData` hook to fetch all activities for the range
    - Compute summaries using `computeFeedingSummary`, `computeSleepSummary`, `computeDiaperSummary`, `computePumpingSummary`
    - Render `TimeRangeSelector`, four `ActivitySummaryCard` components (feeding, sleep, diaper, pumping)
    - Show loading spinner, error message, and global empty state as appropriate
    - Provide "Back to Dependent" navigation link
    - _Requirements: 1.2, 1.3, 2.9, 3.1, 3.2, 3.3, 4.1–4.3, 4.5, 5.1–5.3, 5.5, 6.1, 6.2, 6.4, 7.1–7.3, 7.5, 9.1_

  - [x] 7.2 Register route in `src/App.tsx`
    - Add `/families/:familyId/dependents/:dependentId/reports` route pointing to `ReportPage` wrapped in `ProtectedRoute`
    - _Requirements: 1.4_

  - [x] 7.3 Add "Reports" navigation link to `src/pages/DependentDetailPage.tsx`
    - Add a "Reports" button/link that navigates to the reports route for the current dependent
    - _Requirements: 1.1_

- [ ] 8. Checkpoint - Verify summaries end-to-end
  - Ensure all tests pass, ask the user if questions arise. At this point the feature should be usable: navigate to reports, pick a time range, and see summary stats.

- [ ] 9. Install Recharts and implement ActivityChart component
  - [ ] 9.1 Install Recharts dependency
    - Run `npm install recharts` in `famtrac-frontend`
    - _Requirements: 8.1_

  - [ ] 9.2 Create `src/components/reports/ActivityChart.tsx`
    - Accept `title`, `data`, `chartType` (`'line' | 'bar'`), `yAxisLabel`, `xAxisLabel`, `color`, `emptyMessage` props
    - Render `ResponsiveContainer` wrapping `LineChart` or `BarChart` based on `chartType`
    - Include `XAxis`, `YAxis`, `Tooltip`, and `Line`/`Bar` components
    - When `data` is empty, render `emptyMessage` text instead of chart
    - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [ ] 10. Integrate charts into ReportPage
  - [ ] 10.1 Update `src/pages/ReportPage.tsx` to include charts
    - Compute chart data using `transformFeedingChartData`, `transformSleepChartData`, `transformDiaperChartData`, `transformPumpingChartData`
    - Render four `ActivityChart` components: feeding line, sleep bar, diaper bar, pumping line
    - Pass appropriate empty state messages for each chart type
    - _Requirements: 4.4, 4.6, 5.4, 5.6, 6.3, 6.5, 7.4, 7.6_

- [ ] 11. Component unit tests
  - [ ]* 11.1 Write unit tests for `TimeRangeSelector`
    - Test that three preset buttons and two date inputs render
    - Test clicking a preset calls `onPresetSelect` with correct value
    - Test entering custom dates calls `onCustomRangeChange`
    - Test active preset button has correct visual variant
    - _Requirements: 2.1, 2.5, 2.6, 2.7, 2.8_

  - [ ]* 11.2 Write unit tests for `ActivitySummaryCard`
    - Test that title and all metric label/value pairs render
    - _Requirements: 4.1, 5.1, 6.1, 7.1_

  - [ ]* 11.3 Write unit tests for `ActivityChart`
    - Test that empty message renders when data is empty
    - _Requirements: 4.6, 5.6, 6.5, 7.6_

  - [ ]* 11.4 Write unit tests for `ReportPage`
    - Test loading spinner displays while data loads
    - Test error message displays on API error
    - Test global empty state when no activities exist
    - _Requirements: 3.2, 3.3, 9.1_

- [ ] 12. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- After task 8, the feature is usable end-to-end with summaries (no charts yet)
- Charts (tasks 9–10) layer on as an enhancement using Recharts
- Property tests validate universal correctness properties from the design document
- All code is TypeScript/React, matching the existing codebase
