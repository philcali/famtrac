# Epic: Meal Planning Integration

Integrate Little Eater (food-plan) meal planning capabilities into famtrac, allowing parents to plan meals for a dependent and log feedings that flow into famtrac's activity system.

## Architecture

```
famtrac frontend ──→ famtrac backend ──→ DynamoDB
         │
  MealPlanPage (new)
  Recipe API (new)
  MealSlot API (new)
  FeedingLog API (new)
```

Key principle: **Don't embed food-plan as an iframe.** Build meal planning natively into famtrac so data flows into the existing feeding/activity system.

---

## Story 1: Backend — Recipe Domain Model & CRUD ✅ COMPLETE

**Goal:** Add `Recipe` as a new domain entity in famtrac-backend with full CRUD API.

### Acceptance Criteria

- [x] Add `Recipe` struct in `famtrac-backend/src/domain/` with fields: `id`, `family_id`, `name`, `emoji`, `ingredients[]`, `age_min`, `texture`, `allergens[]`, `prep_notes`, `safe`, `created_at`
- [x] Add `RecipeId` type to `famtrac-backend/src/domain/ids.rs`
- [x] Add `RecipeRepository` trait in `famtrac-backend/src/repository/traits.rs`
- [x] Add `DynamoDbRecipeRepository` impl in `famtrac-backend/src/repository/dynamodb.rs`
- [ ] Add DynamoDB table for recipes (infra update) — deferred to Story 9 (Infra)
- [x] Add routes: `GET/POST /families/{fid}/recipes`, `GET/PUT/DELETE /families/{fid}/recipes/{id}`
- [x] Add request/response types in `famtrac-backend/src/handlers/recipe.rs`
- [x] Add routes to `famtrac-backend/src/router/mod.rs` dispatch logic
- [x] Add `Recipe` to frontend `famtrac-frontend/src/types/domain.ts`
- [x] Add `famtrac-frontend/src/api/recipes.ts` with `list`, `get`, `create`, `update`, `delete` functions
- [x] All new code follows existing patterns (naming, error handling, logging)
- [x] Backend compiles without warnings

### Files changed

| File | Change |
|------|--------|
| `src/domain/ids.rs` | Added `RecipeId(pub Uuid)` |
| `src/domain/recipe.rs` | New file — `Recipe`, `CreateRecipeRequest`, `UpdateRecipeRequest` |
| `src/domain/mod.rs` | Exported `recipe` module |
| `src/repository/traits.rs` | Added `RecipeRepository` trait |
| `src/repository/dynamodb.rs` | Added `DynamoDbRecipeRepository` (~200 lines) |
| `src/repository/mod.rs` | Exported recipe types |
| `src/handlers/recipe.rs` | New file — CRUD handlers + unit tests |
| `src/handlers/mod.rs` | Exported recipe handlers |
| `src/router/recipe.rs` | New file — route dispatch + tests |
| `src/router/family.rs` | Added recipe_repo param + route dispatch |
| `src/router/mod.rs` | Added recipe module + repo param |
| `src/main.rs` | Wired `DynamoDbRecipeRepository` |
| `src/test_utils.rs` | Added `MockRecipeRepository` |
| `frontend/src/types/domain.ts` | Added `Recipe`, `CreateRecipeRequest`, `UpdateRecipeRequest`, `PaginatedResponse` |
| `frontend/src/api/types.ts` | Added API response types |
| `frontend/src/api/recipes.ts` | New file — API client functions |

### Dependencies

- None (foundation story)

---

## Story 2: Backend — MealSlot Domain Model & CRUD ✅ COMPLETE

**Goal:** Add `MealSlot` as a new domain entity in famtrac-backend with full CRUD API.

### Acceptance Criteria

- [x] Add `MealSlot` struct in `famtrac-backend/src/domain/` with fields: `id`, `family_id`, `dependent_id`, `day` (YYYY-MM-DD), `time` (HH:MM), `recipe_id`, `notes`
- [x] Add `MealSlotId` type to `famtrac-backend/src/domain/ids.rs`
- [x] Add `MealSlotRepository` trait in `famtrac-backend/src/repository/traits.rs`
- [x] Add `DynamoDbMealSlotRepository` impl in `famtrac-backend/src/repository/dynamodb.rs`
- [ ] Add DynamoDB table for meal slots (infra update) — deferred to Story 9 (Infra)
- [x] Add routes: `GET/POST /families/{fid}/dependents/{did}/meal-slots`, `GET/PUT/DELETE /families/{fid}/dependents/{did}/meal-slots/{id}`
- [x] Add request/response types in `famtrac-backend/src/router/extractors.rs`
- [x] Add routes to `famtrac-backend/src/router/mod.rs` dispatch logic
- [x] Add `MealSlot` to frontend `famtrac-frontend/src/types/domain.ts`
- [x] Add `famtrac-frontend/src/api/mealSlots.ts` with `list`, `get`, `create`, `update`, `delete` functions
- [x] All new code follows existing patterns
- [x] Backend compiles without warnings

### Files changed

| File | Change |
|------|--------|
| `src/domain/meal_slot.rs` | New file — `MealSlot`, `CreateMealSlotRequest`, `UpdateMealSlotRequest` |
| `src/domain/ids.rs` | Added `MealSlotId(pub Uuid)` |
| `src/domain/mod.rs` | Exported `meal_slot` module |
| `src/repository/traits.rs` | Added `MealSlotRepository` trait |
| `src/repository/dynamodb.rs` | Added `DynamoDbMealSlotRepository` (~300 lines) |
| `src/repository/mod.rs` | Exported meal_slot types |
| `src/handlers/meal_slot.rs` | New file — CRUD handlers (~800 lines) |
| `src/handlers/mod.rs` | Exported meal_slot handlers |
| `src/router/meal_slot.rs` | New file — route dispatch + tests |
| `src/router/dependent.rs` | Added meal_slot_repo param + route dispatch |
| `src/router/mod.rs` | Added meal_slot module + repo param |
| `src/main.rs` | Wired `DynamoDbMealSlotRepository` |
| `src/test_utils.rs` | Added `MockMealSlotRepository` |
| `frontend/src/types/domain.ts` | Added `MealSlot`, `CreateMealSlotRequest`, `UpdateMealSlotRequest` |
| `frontend/src/api/types.ts` | Added API response types |
| `frontend/src/api/mealSlots.ts` | New file — API client functions |

### Dependencies

- Story 1

---

## Story 3: Backend — FeedingLog Domain Model & CRUD ✅ COMPLETE

**Goal:** Add `FeedingLog` as a new domain entity in famtrac-backend with full CRUD API.

### Acceptance Criteria

- [x] Add `FeedingLog` struct in `famtrac-backend/src/domain/` with fields: `id`, `family_id`, `dependent_id`, `date` (YYYY-MM-DD), `time` (HH:MM), `recipe_id`, `amount`, `reaction`, `notes`, `created_at`
- [x] Add `FeedingLogId` type to `famtrac-backend/src/domain/ids.rs`
- [x] Add `FeedingLogRepository` trait in `famtrac-backend/src/repository/traits.rs`
- [x] Add `DynamoDbFeedingLogRepository` impl in `famtrac-backend/src/repository/dynamodb.rs`
- [ ] Add DynamoDB table for feeding logs (infra update) — deferred to Story 9 (Infra)
- [x] Add routes: `GET/POST /families/{fid}/dependents/{did}/feeding-logs`, `GET/PUT/DELETE /families/{fid}/dependents/{did}/feeding-logs/{id}`
- [x] Add request/response types in `famtrac-backend/src/router/extractors.rs`
- [x] Add routes to `famtrac-backend/src/router/mod.rs` dispatch logic
- [x] Add `FeedingLog` to frontend `famtrac-frontend/src/types/domain.ts`
- [x] Add `famtrac-frontend/src/api/feedingLogs.ts` with `list`, `get`, `create`, `update`, `delete` functions
- [x] All new code follows existing patterns
- [x] Backend compiles without warnings

### Files changed

| File | Change |
|------|--------|
| `src/domain/feeding_log.rs` | New file — `FeedingLog`, `CreateFeedingLogRequest`, `UpdateFeedingLogRequest` |
| `src/domain/ids.rs` | Added `FeedingLogId(pub Uuid)` |
| `src/domain/mod.rs` | Exported `feeding_log` module |
| `src/repository/traits.rs` | Added `FeedingLogRepository` trait |
| `src/repository/dynamodb.rs` | Added `DynamoDbFeedingLogRepository` (~400 lines) |
| `src/repository/mod.rs` | Exported feeding_log types |
| `src/handlers/feeding_log.rs` | New file — CRUD handlers (~800 lines) |
| `src/handlers/mod.rs` | Exported feeding_log handlers |
| `src/router/feeding_log.rs` | New file — route dispatch + tests |
| `src/router/dependent.rs` | Added feeding_log_repo param + route dispatch |
| `src/router/mod.rs` | Added feeding_log module + repo param |
| `src/main.rs` | Wired `DynamoDbFeedingLogRepository` |
| `src/test_utils.rs` | Added `MockFeedingLogRepository` |
| `frontend/src/types/domain.ts` | Added `FeedingLog`, `CreateFeedingLogRequest`, `UpdateFeedingLogRequest` |
| `frontend/src/api/types.ts` | Added API response types |
| `frontend/src/api/feedingLogs.ts` | New file — API client functions |

### Dependencies

- Story 2

---

## Story 4: Frontend — Recipe Library Page

**Goal:** Add a "Recipes" page to famtrac-frontend for browsing and managing the family's recipe collection.

### Acceptance Criteria

- [ ] New route: `/families/:familyId/recipes`
- [ ] Page lists all recipes with emoji, name, age, texture, allergen badges
- [ ] Search bar filters recipes by name and ingredients
- [ ] "Add Recipe" button opens a modal form (name, emoji, ingredients, age, texture, allergens, prep notes, safe toggle)
- [ ] Edit and delete actions on each recipe card
- [ ] Uses existing famtrac component patterns (Button, Card, Modal/ConfirmDialog)
- [ ] Tailwind CSS styling matches famtrac design system
- [ ] Loading and empty states handled
- [ ] Error state display

### Dependencies

- Story 1

---

## Story 5: Frontend — Meal Plan Page

**Goal:** Add a "Meal Plan" page to famtrac-frontend with week-by-week meal planning UI.

### Acceptance Criteria

- [ ] New route: `/families/:familyId/dependents/:dependentId/meal-plan`
- [ ] Week navigation (prev/next arrows, date picker, "Go to Today")
- [ ] Day-of-week tabs (Mon–Sun) with meal count indicators
- [ ] Selected day shows meal slot cards sorted by time
- [ ] Each slot card shows: recipe emoji/name, age, texture, allergen warnings, time picker, notes
- [ ] "Add recipe" empty slot button per day
- [ ] "Add Recipe" modal: search + select from recipe library
- [ ] "Edit Slot" modal: change time, swap recipe, add notes
- [ ] "Log Feeding" modal: date, time, recipe, amount eaten, reaction, notes
- [ ] Delete confirmation on slot removal
- [ ] Uses famtrac component patterns (Button, Card, Modal/ConfirmDialog)
- [ ] Tailwind CSS styling matches famtrac design system
- [ ] Loading and empty states handled
- [ ] Error state display

### Dependencies

- Story 1, Story 2, Story 4

---

## Story 6: Data Bridge — Meal Plan → Feeding Activity

**Goal:** When a user logs a feeding from the meal plan, create a famtrac `feeding` activity so the data flows into reports/analytics.

### Acceptance Criteria

- [ ] "Log Feeding" from meal plan creates a `feeding` activity via the existing famtrac Activity API
- [ ] `feeding_type` set to `'solid'`
- [ ] `volume_ml` derived from amount (Tasted=10, Ate some=30, Ate most=60, Ate all=90, Refused=0)
- [ ] `notes` from the feeding log carried over to the activity
- [ ] `timestamp` set to the logged date/time
- [ ] After logging, show success message and refresh activity list
- [ ] Feeding log data is also persisted via the new FeedingLog API for historical reference
- [ ] Both the activity and the feeding log are created atomically (or with clear error handling if one fails)

### Dependencies

- Story 3, Story 5

---

## Story 7: Navigation — Meal Plan Entry Points

**Goal:** Add navigation links to the meal plan from the dependent detail page and the family detail page.

### Acceptance Criteria

- [ ] "Meal Plan" button/link on `DependentDetailPage` heading
- [ ] "Meal Plan" link on `FamilyDetailPage` dependent cards
- [ ] "Back to Dependent" link on Meal Plan page
- [ ] "Recipes" link on Meal Plan page
- [ ] Routes use famtrac's existing `useNavigate` pattern

### Dependencies

- Story 4, Story 5

---

## Story 8: Optional — Food-Plan Import

**Goal:** Allow users to import recipes and feeding logs from a Little Eater (food-plan) export JSON file.

### Acceptance Criteria

- [ ] "Import" button on Recipe Library page
- [ ] File picker accepts Little Eater export JSON
- [ ] Validates export format (version 1, type: "recipes" or "full")
- [ ] Shows preview: recipes count, feeding logs count
- [ ] "Import recipes" creates recipes in famtrac
- [ ] "Import feeding logs" creates feeding activities in famtrac
- [ ] Success/error feedback
- [ ] Import is additive (doesn't delete existing data)

### Dependencies

- Story 1, Story 3, Story 4

---

## Story 9: Infrastructure — DynamoDB Table Schema

**Goal:** Update famtrac's single-table DynamoDB schema to support Recipe, MealSlot, and FeedingLog item types.

### Acceptance Criteria

- [ ] Verify existing DynamoDB table schema supports new `Type` discriminator values (Recipe, MealSlot, FeedingLog)
- [ ] Update IAM policy to allow access to all item types in the table
- [ ] Update CDK/Terraform config if any table-level changes are needed (GSI, billing mode, etc.)
- [ ] Run `cdk synth` / `terraform plan` to confirm no unexpected infrastructure changes
- [ ] Document the single-table key schema for the new entity types

### Dependencies

- None (can be done anytime, but recommended before production deployment)

---

## Infrastructure Changes

| Item | Details |
|------|---------|
| DynamoDB tables | `recipes`, `meal_slots`, `feeding_logs` |
| Table keys | Partition: `family_id`, Sort: `dependent_id` (where applicable) |
| Terraform/CDK | Update famtrac-infra to provision new tables |
| Permissions | Update IAM policy for new table access |

> **Note:** Story 1's infra item is deferred to Story 9 (Infra) since famtrac uses a single DynamoDB table design. New item types are added to the existing table via the `Type` discriminator, so no new table is needed.

---

## Implementation Order

```
Story 1 (Recipes) ✅
    └── Story 2 (MealSlots) ✅
            └── Story 3 (FeedingLogs) ✅
                    └── Story 4 (Recipe Library Page)
                    └── Story 5 (Meal Plan Page)
                            └── Story 6 (Data Bridge)
                            └── Story 7 (Navigation)
                                    └── Story 8 (Import)

Story 9 (Infra) — can run in parallel with Stories 2–9
```

Stories 4 and 5 can be developed in parallel once Story 3 is complete. Story 9 (Infra) can be done anytime but should precede production deployment.