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
- [x] Add DynamoDB table for recipes (infra update) — deferred to Story 9 (Infra) → no infra changes needed
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
- [x] Add DynamoDB table for meal slots (infra update) — deferred to Story 9 (Infra) → no infra changes needed
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
- [x] Add DynamoDB table for feeding logs (infra update) — deferred to Story 9 (Infra) → no infra changes needed
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

## Story 4: Frontend — Recipe Library Page ✅ COMPLETE

**Goal:** Add a "Recipes" page to famtrac-frontend for browsing and managing the family's recipe collection.

### Acceptance Criteria

- [x] New route: `/families/:familyId/recipes`
- [x] Page lists all recipes with emoji, name, age, texture, allergen badges
- [x] Search bar filters recipes by name and ingredients
- [x] "Add Recipe" button opens a modal form (name, emoji, ingredients, age, texture, allergens, prep notes, safe toggle)
- [x] Edit and delete actions on each recipe card
- [x] Uses existing famtrac component patterns (Button, Card, Modal/ConfirmDialog)
- [x] Tailwind CSS styling matches famtrac design system
- [x] Loading and empty states handled
- [x] Error state display

### Dependencies

- Story 1

### Files changed

| File | Change |
|------|--------|
| `famtrac-frontend/src/pages/RecipeLibraryPage.tsx` | New file — Recipe library page with search, CRUD modals, loading/empty/error states |
| `famtrac-frontend/src/components/recipes/RecipeCard.tsx` | New file — Recipe card with emoji, name, age, texture, allergen badges |
| `famtrac-frontend/src/components/recipes/RecipeForm.tsx` | New file — Modal form for create/edit (name, emoji, ingredients, age, texture, allergens, prep notes, safe toggle) |
| `famtrac-frontend/src/App.tsx` | Added `/families/:familyId/recipes` route |

---

## Story 5: Frontend — Meal Plan Page ✅ COMPLETE

**Goal:** Add a "Meal Plan" page to famtrac-frontend with week-by-week meal planning UI.

### Acceptance Criteria

- [x] New route: `/families/:familyId/dependents/:dependentId/meal-plan`
- [x] Week navigation (prev/next arrows, date picker, "Go to Today")
- [x] Day-of-week tabs (Mon–Sun) with meal count indicators
- [x] Selected day shows meal slot cards sorted by time
- [x] Each slot card shows: recipe emoji/name, age, texture, allergen warnings, time picker, notes
- [x] "Add recipe" empty slot button per day
- [x] "Add Recipe" modal: search + select from recipe library
- [x] "Edit Slot" modal: change time, swap recipe, add notes
- [x] "Log Feeding" modal: date, time, recipe, amount eaten, reaction, notes
- [x] Delete confirmation on slot removal
- [x] Uses famtrac component patterns (Button, Card, Modal/ConfirmDialog)
- [x] Tailwind CSS styling matches famtrac design system
- [x] Loading and empty states handled
- [x] Error state display

### Files changed

| File | Change |
|------|--------|
| `famtrac-frontend/src/pages/MealPlanPage.tsx` | New file — week-by-week meal planning UI with navigation, day tabs, slot cards, loading/empty/error states |
| `famtrac-frontend/src/components/meals/MealSlotCard.tsx` | New file — meal slot card with emoji, name, age, texture, allergen badges, edit/log/delete actions |
| `famtrac-frontend/src/components/meals/RecipePickerModal.tsx` | New file — searchable recipe picker for add/edit slot |
| `famtrac-frontend/src/components/meals/FeedingLogModal.tsx` | New file — feeding log form (date, time, recipe, amount, reaction, notes) |
| `famtrac-frontend/src/App.tsx` | Added `/families/:familyId/dependents/:dependentId/meal-plan` route |
| `famtrac-frontend/src/components/recipes/RecipeCard.tsx` | Minor update (spacing tweak) |

### Dependencies

- Story 1, Story 2, Story 4

---

## Story 6: Data Bridge — Meal Plan → Feeding Activity ✅ COMPLETE

**Goal:** When a user logs a feeding from the meal plan, create a famtrac `feeding` activity so the data flows into reports/analytics.

### Acceptance Criteria

- [x] "Log Feeding" from meal plan creates a `feeding` activity via the existing famtrac Activity API
- [x] `feeding_type` set to `'solid'`
- [x] `volume_ml` derived from amount (Tasted=10, Ate some=30, Ate most=60, Ate all=90, Refused=0)
- [x] `notes` from the feeding log carried over to the activity
- [x] `timestamp` set to the logged date/time
- [x] After logging, show success message and refresh activity list
- [x] Feeding log data is also persisted via the new FeedingLog API for historical reference
- [x] Both the activity and the feeding log are created atomically (or with clear error handling if one fails)

### Dependencies

- Story 3, Story 5

### Files changed

| File | Change |
|------|--------|
| `famtrac-frontend/src/pages/MealPlanPage.tsx` | Added `createActivity` import + `createActivityMutation` hook; modified `handleFeedingSubmit` to also create a `feeding` activity with `feeding_type='solid'`, `volume_ml` mapped from reaction, `notes` carried over, `timestamp` from logged date/time; added `toISOWithOffset` helper |
| `famtrac-frontend/src/api/activities.ts` | No changes (existing `createActivity` used as-is) |

---

## Story 7: Navigation — Meal Plan Entry Points ✅ COMPLETE

**Goal:** Add navigation links to the meal plan from the dependent detail page and the family detail page.

### Acceptance Criteria

- [x] "Meal Plan" button/link on `DependentDetailPage` heading
- [x] "Meal Plan" link on `FamilyDetailPage` dependent cards
- [x] "Back to Dependent" link on Meal Plan page
- [x] "Recipes" link on Meal Plan page
- [x] Routes use famtrac's existing `useNavigate` pattern

### Dependencies

- Story 4, Story 5

### Files changed

| File | Change |
|------|--------|
| `famtrac-frontend/src/pages/DependentDetailPage.tsx` | Added "Meal Plan" button in heading, navigates to `/families/:familyId/dependents/:dependentId/meal-plan` |
| `famtrac-frontend/src/pages/FamilyDetailPage.tsx` | Added `handleMealPlanClick` handler + wired to `DependentList.onMealPlan` |
| `famtrac-frontend/src/components/dependents/DependentCard.tsx` | Added `onMealPlan` prop + calendar icon button |
| `famtrac-frontend/src/components/dependents/DependentList.tsx` | Added `onMealPlan` prop, passed through to `DependentCard` |
| `famtrac-frontend/src/pages/MealPlanPage.tsx` | Changed "← Back" → "← Back to Dependent" (explicit nav), added "Recipes" link in header/loading/error states |
| `famtrac-frontend/src/utils/iconRegistry.ts` | Added `calendar` icon SVG |

---

## Story 8: Optional — Food-Plan Import ✅ COMPLETE

**Goal:** Allow users to import recipes and feeding logs from a Little Eater (food-plan) export JSON file.

### Acceptance Criteria

- [x] "Import" button on Recipe Library page
- [x] File picker accepts Little Eater export JSON
- [x] Validates export format (version 1, type: "recipes" or "full")
- [x] Shows preview: recipes count, feeding logs count
- [x] "Import recipes" creates recipes in famtrac
- [x] "Import feeding logs" creates feeding activities in famtrac
- [x] Success/error feedback
- [x] Import is additive (doesn't delete existing data)

### Files changed

| File | Change |
|------|--------|
| `famtrac-frontend/src/components/recipes/ImportModal.tsx` | New file — multi-step import modal (preview → import recipes → import feeding logs → results) |
| `famtrac-frontend/src/pages/RecipeLibraryPage.tsx` | Added "Import" button, import modal integration, success message handling |
| `famtrac-frontend/src/types/domain.ts` | Added `LittleEaterExport`, `LittleEaterRecipe`, `LittleEaterFeedingLog` types |

### Dependencies

- Story 1, Story 3, Story 4

---

## Story 10: Backend — Share Recipes, MealSlots, and FeedingLogs via Stream Handler ✅ COMPLETE

**Goal:** Extend the famtrac-stream-handler to mirror and propagate Recipe, MealSlot, and FeedingLog records so that shared families include meal planning data.

### Background

The stream handler currently mirrors only Family, Dependent, and Activity records. Stories 1–3 added Recipe, MealSlot, and FeedingLog domain models with `family_id`, `share_id`, and `permission_scope` fields. Without stream handler support, a shared family's recipes, meal plans, and feeding logs remain invisible to the accepter.

**Current mirror scope:** Family (rekeyed), Dependent (annotated), Activity (annotated).

**Missing mirror scope:** Recipe, MealSlot, FeedingLog — all have `family_id` (indexed by `GSI-family_id`) and already carry `share_id`/`permission_scope` fields on the domain structs.

### Acceptance Criteria

#### Share Activation (mirror on accept)

- [x] WHEN a share transitions to `active`, the stream handler mirrors all Recipe records for the family into the accepter's partition with a rekeyed PK of `OWNER#{accepter_id}` and SK `RECIPE#{recipe_id}`
- [x] WHEN a share transitions to `active`, the stream handler mirrors all MealSlot records for the family's dependents, annotated with `share_id` and `permission_scope` (same PK/SK as originals)
- [x] WHEN a share transitions to `active`, the stream handler mirrors all FeedingLog records for the family's dependents, annotated with `share_id` and `permission_scope` (same PK/SK as originals)
- [x] Every mirrored item is stamped with `sync_token` to identify it as handler-originated

#### Resource Change Propagation (propagate on CRUD)

- [x] WHEN a Recipe is created/updated/deleted on the owner, the change propagates to all mirrored Recipe copies in accepter partitions
- [x] WHEN a MealSlot is created/updated/deleted on the owner, the change propagates to all mirrored MealSlot copies in accepter partitions
- [x] WHEN a FeedingLog is created/updated/deleted on the owner, the change propagates to all mirrored FeedingLog copies in accepter partitions
- [x] The classifier recognizes `RECIPE#`, `MEAL_SLOT#`, and `FEEDING_LOG#` SK prefixes and classifies them as `ResourceChanged`
- [x] `extract_family_id` handles Recipe PKs (`FAMILY#{fid}`) and MealSlot/FeedingLog PKs (`FAMILY#{fid}#DEPENDENT#{did}`) via fallback parsing

#### Write-back from Mirrors (accepter writes)

- [x] WHEN an accepter writes to a mirrored Recipe, the change propagates back to the owner's partition
- [x] WHEN an accepter writes to a mirrored MealSlot or FeedingLog, the share metadata is preserved on the item
- [x] Semantic diff prevents no-op write-backs (identical items are not re-written)

#### Share Revocation (cleanup on revoke)

- [x] WHEN a share is revoked, the stream handler deletes the mirrored Recipe records from the accepter's partition
- [x] WHEN a share is revoked, the stream handler deletes all mirrored MealSlot records annotated with the revoked `share_id`
- [x] WHEN a share is revoked, the stream handler deletes all mirrored FeedingLog records annotated with the revoked `share_id`

#### Permission Scope Updates

- [x] WHEN a share's permission scope is updated, the `permission_scope` is updated on the mirrored Family record in the accepter's partition
- [x] WHEN a share's permission scope is updated, the `permission_scope` is updated on all mirrored Recipe records with the matching `share_id`
- [x] WHEN a share's permission scope is updated, the `permission_scope` is updated on all mirrored MealSlot records with the matching `share_id`
- [x] WHEN a share's permission scope is updated, the `permission_scope` is updated on all mirrored FeedingLog records with the matching `share_id`
- [x] Each update uses a condition expression `share_id = :sid` so that only mirrored copies (not originals) are affected
- [x] ConditionalCheckFailedExceptions are silently ignored (the record may not exist or may not be mirrored)

### Technical Notes

- **Recipe PK pattern:** `FAMILY#{family_id}` / `RECIPE#{recipe_id}` — requires rekeying into accepter's OWNER partition (like Family)
- **MealSlot PK pattern:** `FAMILY#{family_id}#DEPENDENT#{dependent_id}` / `MEAL_SLOT#{meal_slot_id}` — same PK/SK, annotated (like Activity)
- **FeedingLog PK pattern:** `FAMILY#{family_id}#DEPENDENT#{dependent_id}` / `FEEDING_LOG#{feeding_log_id}` — same PK/SK, annotated (like Activity)
- No new GSI is needed — all three types already have `family_id` and are indexed by `GSI-family_id`
- The domain structs (`Recipe`, `MealSlot`, `FeedingLog`) already have `share_id: Option<ShareId>` and `permission_scope: Option<PermissionScope>` fields

### Files changed

| File | Change |
|------|--------|
| `famtrac-stream-handler/src/classify.rs` | Added `RECIPE#`, `MEAL_SLOT#`, `FEEDING_LOG#` to `record_type_from_sk()` + classifier match arm; added unit tests |
| `famtrac-stream-handler/src/dynamo_util.rs` | Extended `extract_family_id()` to handle Recipe PK (`FAMILY#{fid}`) and MealSlot/FeedingLog PK (`FAMILY#{fid}#DEPENDENT#{did}`) fallback parsing; added unit tests |
| `famtrac-stream-handler/src/handlers/mirror.rs` | Added mirror steps for Recipe (rekeyed), MealSlot (annotated), FeedingLog (annotated) on share activation |
| `famtrac-stream-handler/src/handlers/revoke.rs` | Added deletion of mirrored Recipe, MealSlot, FeedingLog records on share revocation |
| `famtrac-stream-handler/src/handlers/propagate.rs` | Added Recipe write-back (rekeyed), MealSlot/FeedingLog write-back with share metadata preservation and semantic diff |
| `famtrac-stream-handler/src/handlers/permission.rs` | Added permission_scope updates on mirrored Recipe, MealSlot, FeedingLog records |
| `famtrac-backend/src/domain/share.rs` | Added ShareId, PermissionScope domain types |
| `famtrac-frontend/src/types/domain.ts` | Updated share-related types |
| `famtrac-frontend/src/utils/permissions.ts` | Added permission scope utilities |
| `famtrac-frontend/src/components/shares/PermissionScopeSelector.tsx` | Updated selector for meal planning context |

### Dependencies

- Story 1 (Recipe domain model)
- Story 2 (MealSlot domain model)
- Story 3 (FeedingLog domain model)

---

## Story 9: Infrastructure — DynamoDB Table Schema ✅ COMPLETE

**Goal:** Update famtrac's single-table DynamoDB schema to support Recipe, MealSlot, and FeedingLog item types.

### Acceptance Criteria

- [x] Verify existing DynamoDB table schema supports new `Type` discriminator values (Recipe, MealSlot, FeedingLog)
- [x] Update IAM policy to allow access to all item types in the table
- [x] Update CDK/Terraform config if any table-level changes are needed (GSI, billing mode, etc.)
- [x] Run `cdk synth` / `terraform plan` to confirm no unexpected infrastructure changes
- [x] Document the single-table key schema for the new entity types

### Dependencies

- None (can be done anytime, but recommended before production deployment)

### Notes

No infrastructure changes required. Famtrac uses a single DynamoDB table with a `Type` discriminator — adding new `Type` values (Recipe, MealSlot, FeedingLog) needs zero schema, IAM, or CDK modifications. Key schema patterns are documented in `famtrac-infra/lib/backend/FamtracApi.ts`.

---

## Infrastructure Changes

| Item | Details |
|------|---------|
| DynamoDB table | Single `FamtracData` table (no new tables) |
| New item types | `Recipe`, `MealSlot`, `FeedingLog` — added via `Type` discriminator |
| Terraform/CDK | No changes needed (Story 9 complete) |
| Permissions | Already covers all item types (table-level CRUD) |

---

## Implementation Order

```
Story 1 (Recipes) ✅
    └── Story 2 (MealSlots) ✅
            └── Story 3 (FeedingLogs) ✅
                    └── Story 4 (Recipe Library Page) ✅
                    └── Story 5 (Meal Plan Page) ✅
                            └── Story 6 (Data Bridge) ✅
                            └── Story 7 (Navigation) ✅
                                    └── Story 8 (Import) ✅

Story 10 (Stream: share recipes/meal-slots/feeding-logs) ✅ — ran in parallel with Stories 4–9

Story 9 (Infra) — can run in parallel with Stories 2–10
```

Stories 4 and 5 can be developed in parallel once Story 3 is complete. Story 9 (Infra) can be done anytime but should precede production deployment.