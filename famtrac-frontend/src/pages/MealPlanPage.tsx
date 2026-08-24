import { useState, useMemo, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Button } from '../components/common/Button';
import { ConfirmDialog } from '../components/common/ConfirmDialog';
import { SuccessMessage } from '../components/common/SuccessMessage';
import { ErrorMessage } from '../components/common/ErrorMessage';
import { SkeletonCard } from '../components/common/SkeletonCard';
import { MealSlotCard } from '../components/meals/MealSlotCard';
import { RecipePickerModal } from '../components/meals/RecipePickerModal';
import { FeedingLogModal } from '../components/meals/FeedingLogModal';
import { useAuth } from '../auth/useAuth';
import { useApi, useApiMutation } from '../hooks/useApi';
import { createApiClient } from '../api/client';
import { getRecipes } from '../api/recipes';
import { getMealSlots, createMealSlot, updateMealSlot, deleteMealSlot } from '../api/mealSlots';
import { createFeedingLog } from '../api/feedingLogs';
import { createActivity } from '../api/activities';
import type { RecipeResponse } from '../api/types';
import type { MealSlot, CreateMealSlotRequest, UpdateMealSlotRequest } from '../types/domain';
import type { CreateFeedingLogRequest } from '../types/domain';
import type { CreateActivityRequest } from '../api/types';

const DAY_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

/**
 * MealPlanPage — Week-by-week meal planning UI for a dependent.
 */
export function MealPlanPage() {
  const { familyId, dependentId } = useParams<{ familyId: string; dependentId: string }>();
  const navigate = useNavigate();
  const { getToken } = useAuth();

  const apiClient = createApiClient(getToken);

  // ---- Data fetching ----
  const {
    data: recipesData,
    loading: recipesLoading,
    error: recipesError,
  } = useApi(() => getRecipes(apiClient, familyId ?? 'NA'), [familyId]);

  const {
    data: slotsData,
    loading: slotsLoading,
    error: slotsError,
    refetch: refetchSlots,
  } = useApi(
    () => getMealSlots(apiClient, familyId ?? 'NA', dependentId ?? 'NA'),
    [familyId, dependentId]
  );

  // ---- Local state ----
  const [currentWeekStart, setCurrentWeekStart] = useState(() => getMonday(new Date()));
  const [selectedDay, setSelectedDay] = useState(() => formatDate(new Date()));
  const [showAddRecipe, setShowAddRecipe] = useState(false);
  const [editingSlot, setEditingSlot] = useState<MealSlot | undefined>();
  const [deletingSlot, setDeletingSlot] = useState<MealSlot | undefined>();
  const [feedingSlot, setFeedingSlot] = useState<MealSlot | undefined>();
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [editFormTime, setEditFormTime] = useState('');
  const [editFormNotes, setEditFormNotes] = useState('');

  // ---- Mutations ----
  const { mutate: createSlotMutation } = useApiMutation((data: CreateMealSlotRequest) =>
    createMealSlot(apiClient, familyId ?? 'NA', dependentId ?? 'NA', data)
  );

  const { mutate: updateSlotMutation } = useApiMutation((data: UpdateMealSlotRequest) =>
    updateMealSlot(apiClient, familyId ?? 'NA', dependentId ?? 'NA', editingSlot!.id, data)
  );

  const { mutate: deleteSlotMutation, loading: deleteSlotLoading } = useApiMutation((id: string) =>
    deleteMealSlot(apiClient, familyId ?? 'NA', dependentId ?? 'NA', id)
  );

  const { mutate: createFeedingLogMutation, loading: feedingLogLoading } = useApiMutation(
    (data: CreateFeedingLogRequest) =>
      createFeedingLog(apiClient, familyId ?? 'NA', dependentId ?? 'NA', data)
  );

  const { mutate: createActivityMutation, loading: activityLoading } = useApiMutation(
    (data: CreateActivityRequest) =>
      createActivity(apiClient, familyId ?? 'NA', dependentId ?? 'NA', data)
  );

  // ---- Derived data ----
  const recipes: RecipeResponse[] = useMemo(() => recipesData?.recipes ?? [], [recipesData]);
  const slots = useMemo(() => slotsData?.meal_slots ?? [], [slotsData]);

  // Recipe lookup map for slot cards
  const recipeMap = useMemo(() => {
    const m: Record<string, RecipeResponse> = {};
    recipes.forEach((r) => (m[r.id] = r));
    return m;
  }, [recipes]);

  // Slots for the selected day, sorted by time
  const daySlots = useMemo(
    () => slots.filter((s) => s.day === selectedDay).sort((a, b) => a.time.localeCompare(b.time)),
    [slots, selectedDay]
  );

  // Meal count per day for the current week
  const weekDays = useMemo(() => {
    const days: { date: string; label: string; count: number }[] = [];
    for (let i = 0; i < 7; i++) {
      const d = new Date(currentWeekStart);
      d.setDate(d.getDate() + i);
      const dateStr = formatDate(d);
      const count = slots.filter((s) => s.day === dateStr).length;
      days.push({
        date: dateStr,
        label: DAY_LABELS[i],
        count,
      });
    }
    return days;
  }, [currentWeekStart, slots]);

  // ---- Week navigation ----
  const goToPrevWeek = () => {
    const d = new Date(currentWeekStart);
    d.setDate(d.getDate() - 7);
    setCurrentWeekStart(d);
  };

  const goToNextWeek = () => {
    const d = new Date(currentWeekStart);
    d.setDate(d.getDate() + 7);
    setCurrentWeekStart(d);
  };

  const goToToday = () => {
    setCurrentWeekStart(getMonday(new Date()));
    setSelectedDay(formatDate(new Date()));
  };

  // ---- Slot handlers ----
  const handleAddSlot = () => {
    setEditingSlot(undefined);
    setEditFormTime('08:00');
    setEditFormNotes('');
    setShowAddRecipe(true);
  };

  const handleSelectRecipe = async (recipe: RecipeResponse) => {
    if (editingSlot) {
      const response = await updateSlotMutation({
        day: editingSlot.day,
        time: editFormTime,
        recipe_id: recipe.id,
        notes: editFormNotes || undefined,
      });
      if (!response.error) {
        setSuccessMessage('Meal slot updated');
        setEditingSlot(undefined);
        setShowAddRecipe(false);
        refetchSlots();
      }
      return;
    }

    // Create new slot
    const response = await createSlotMutation({
      family_id: familyId ?? 'NA',
      dependent_id: dependentId ?? 'NA',
      day: selectedDay,
      time: editFormTime,
      recipe_id: recipe.id,
      notes: editFormNotes,
    });
    if (!response.error) {
      setSuccessMessage('Meal added to plan');
      setShowAddRecipe(false);
      setEditingSlot(undefined);
      refetchSlots();
    }
  };

  const handleEditSlot = (slotId: string) => {
    const slot = slots.find((s) => s.id === slotId);
    if (!slot) return;
    setEditingSlot(slot);
    setEditFormTime(slot.time);
    setEditFormNotes(slot.notes ?? '');
    setShowAddRecipe(true);
  };

  const handleLogFeeding = (slotId: string) => {
    const slot = slots.find((s) => s.id === slotId);
    if (!slot) return;
    setFeedingSlot(slot);
  };

  const handleFeedingSubmit = async (data: {
    date: string;
    time: string;
    recipe_id: string;
    amount: number;
    reaction: string;
    notes: string;
  }) => {
    if (!feedingSlot) return;

    // Create FeedingLog record
    const feedingLogResponse = await createFeedingLogMutation({
      family_id: familyId ?? 'NA',
      dependent_id: dependentId ?? 'NA',
      date: data.date,
      time: data.time,
      recipe_id: data.recipe_id,
      amount: data.amount,
      reaction: data.reaction,
      notes: data.notes,
    });
    if (feedingLogResponse.error) {
      setSuccessMessage('Failed to log feeding');
      setFeedingSlot(undefined);
      return;
    }

    // Map reaction → volume_ml for the activity (per Story 6 spec)
    const reactionToVolume: Record<string, number> = {
      tasted: 10,
      some: 30,
      most: 60,
      all: 90,
      refused: 0,
    };

    // Build ISO timestamp from logged date + time
    const [y, m, d] = data.date.split('-').map(Number);
    const [h, min] = data.time.split(':').map(Number);
    const local = new Date(y, m - 1, d, h, min, 0);
    const timestamp = toISOWithOffset(local);

    // Look up the recipe name and reaction label for the activity notes
    const activityRecipe = recipes.find((r) => r.id === data.recipe_id);
    const mealSlotRecipe = feedingSlot.recipe_id ? recipeMap[feedingSlot.recipe_id] : undefined;
    const recipeName = activityRecipe?.name ?? mealSlotRecipe?.name;

    const reactionLabel: Record<string, string> = {
      tasted: 'Tasted',
      some: 'Ate some',
      most: 'Ate most',
      all: 'Ate all',
      refused: 'Refused',
    };
    const label = reactionLabel[data.reaction] ?? data.reaction;

    // Build activity notes that reference the meal slot recipe when available
    const activityNotes = recipeName
      ? data.notes
        ? `${label} ${recipeName}: ${data.notes}`
        : `${label} ${recipeName}`
      : data.notes;

    // Create feeding activity so data flows into reports/analytics
    const activityResponse = await createActivityMutation({
      family_id: familyId ?? 'NA',
      dependent_id: dependentId ?? 'NA',
      type: 'feeding',
      timestamp,
      feeding_type: 'solid',
      volume_ml: reactionToVolume[data.reaction] ?? data.amount,
      notes: activityNotes || undefined,
    });
    if (activityResponse.error) {
      setSuccessMessage('Feeding logged but activity creation failed — please retry');
      setFeedingSlot(undefined);
      return;
    }

    setSuccessMessage('Feeding logged & activity created');
    setFeedingSlot(undefined);
    refetchSlots();
  };

  const handleDeleteSlot = () => {
    if (!deletingSlot) return;
    deleteSlotMutation(deletingSlot.id);
    setSuccessMessage('Meal removed from plan');
    setDeletingSlot(undefined);
    refetchSlots();
  };

  const handleBackClick = () => navigate(`/families/${familyId}/dependents/${dependentId}`);
  const handleSuccessClose = useCallback(() => setSuccessMessage(null), []);

  // ---- Loading state ----
  if (recipesLoading || slotsLoading) {
    return (
      <div className="py-4 max-w-5xl mx-auto px-4">
        <div className="mb-4">
          <h2 className="heading">
            Meal Plan
            <div className="ml-auto flex items-center gap-2">
              <Button variant="secondary" onClick={() => navigate(`/families/${familyId}/recipes`)}>
                Recipes
              </Button>
              <Button variant="secondary" onClick={handleBackClick} className="heading-right">
                ← Back to Dependent
              </Button>
            </div>
          </h2>
        </div>
        <SkeletonCard count={4} />
      </div>
    );
  }

  // ---- Error state ----
  if (recipesError || slotsError) {
    return (
      <div className="py-4 max-w-5xl mx-auto px-4">
        <div className="mb-4">
          <h2 className="heading">
            Meal Plan
            <div className="ml-auto flex items-center gap-2">
              <Button variant="secondary" onClick={() => navigate(`/families/${familyId}/recipes`)}>
                Recipes
              </Button>
              <Button variant="secondary" onClick={handleBackClick} className="heading-right">
                ← Back to Dependent
              </Button>
            </div>
          </h2>
        </div>
        <ErrorMessage message={recipesError ?? slotsError ?? 'An error occurred'} />
        <Button onClick={handleBackClick} className="mt-3">
          ← Back to Dependent
        </Button>
      </div>
    );
  }

  // ---- Render ----
  return (
    <div className="py-4 max-w-5xl mx-auto px-4">
      {/* Header */}
      <div className="mb-4">
        <h2 className="heading">
          Meal Plan
          <div className="ml-auto flex items-center gap-2">
            <Button variant="secondary" onClick={() => navigate(`/families/${familyId}/recipes`)}>
              Recipes
            </Button>
            <Button variant="secondary" onClick={handleBackClick} className="heading-right">
              ← Back to Dependent
            </Button>
          </div>
        </h2>
      </div>

      {/* Week navigation */}
      <div className="flex items-center justify-between mb-4">
        <Button variant="secondary" size="sm" onClick={goToPrevWeek}>
          ← Prev
        </Button>
        <div className="text-sm text-gray-600">{formatWeekRange(currentWeekStart)}</div>
        <div className="flex gap-2">
          <Button variant="secondary" size="sm" onClick={goToToday}>
            Today
          </Button>
          <Button variant="secondary" size="sm" onClick={goToNextWeek}>
            Next →
          </Button>
        </div>
      </div>

      {/* Day tabs */}
      <div className="flex gap-1 mb-4 overflow-x-auto">
        {weekDays.map((day) => {
          const isSelected = day.date === selectedDay;
          const isToday = day.date === formatDate(new Date());
          return (
            <button
              key={day.date}
              onClick={() => setSelectedDay(day.date)}
              className={`flex flex-col items-center px-3 py-2 rounded-xl text-sm font-medium transition-colors flex-1 min-w-[60px] ${
                isSelected
                  ? 'bg-blue-500 text-white shadow-sm'
                  : isToday
                    ? 'bg-blue-100 text-blue-700'
                    : 'bg-gray-50 text-gray-600 hover:bg-gray-100'
              }`}
            >
              <span className="text-xs opacity-75">{day.label}</span>
              <span className="text-lg font-bold">{getDayOfMonth(day.date)}</span>
              {day.count > 0 && (
                <span
                  className={`text-[10px] mt-0.5 ${isSelected ? 'text-blue-100' : 'text-gray-400'}`}
                >
                  {day.count}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {/* Success message */}
      {successMessage && <SuccessMessage message={successMessage} onClose={handleSuccessClose} />}

      {/* Selected day content */}
      <div className="mb-4">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-lg font-semibold text-gray-900">
            {selectedDay === formatDate(new Date()) ? 'Today' : `Plan for ${selectedDay}`}
          </h3>
        </div>

        {daySlots.length === 0 ? (
          <div className="text-center py-12 bg-white rounded-xl border border-gray-100">
            <span className="text-4xl">📋</span>
            <h3 className="text-lg font-semibold text-gray-900 mt-3">No meals planned</h3>
            <p className="text-sm text-gray-500 mt-1">Add a recipe to get started.</p>
            <Button className="mt-4" icon="plus" onClick={handleAddSlot}>
              Add Recipe
            </Button>
          </div>
        ) : (
          <div className="space-y-3">
            {daySlots.map((slot) => {
              const recipe = slot.recipe_id ? recipeMap[slot.recipe_id] : undefined;
              return (
                <MealSlotCard
                  key={slot.id}
                  slotId={slot.id}
                  time={slot.time}
                  recipeEmoji={recipe?.emoji}
                  recipeName={recipe?.name}
                  recipeAgeMin={recipe?.age_min}
                  recipeTexture={recipe?.texture}
                  recipeAllergens={recipe?.allergens}
                  recipeSafe={recipe?.safe}
                  notes={slot.notes}
                  onEdit={handleEditSlot}
                  onLogFeeding={handleLogFeeding}
                  onDelete={(id) => setDeletingSlot(slots.find((s) => s.id === id))}
                />
              );
            })}
          </div>
        )}

        {/* Add recipe button (always visible) */}
        <div className="mt-3">
          <Button
            variant="outline-secondary"
            className="w-full"
            icon="plus"
            onClick={handleAddSlot}
          >
            Add Recipe
          </Button>
        </div>
      </div>

      {/* Add/Edit Recipe Modal */}
      <RecipePickerModal
        show={showAddRecipe}
        recipes={recipes}
        onSelect={handleSelectRecipe}
        onClose={() => {
          setShowAddRecipe(false);
          setEditingSlot(undefined);
        }}
      />

      {/* Log Feeding Modal */}
      {feedingSlot && (
        <FeedingLogModal
          show={!!feedingSlot}
          slotDate={feedingSlot.day}
          slotTime={feedingSlot.time}
          recipeId={feedingSlot.recipe_id}
          recipes={recipes}
          onClose={() => setFeedingSlot(undefined)}
          onSubmit={handleFeedingSubmit}
          loading={feedingLogLoading || activityLoading}
        />
      )}

      {/* Delete Confirmation */}
      <ConfirmDialog
        show={!!deletingSlot}
        title="Remove Meal"
        message={`Are you sure you want to remove this meal from the plan?`}
        confirmText="Remove"
        cancelText="Cancel"
        confirmVariant="danger"
        onConfirm={handleDeleteSlot}
        onCancel={() => setDeletingSlot(undefined)}
        loading={deleteSlotLoading}
      />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function getMonday(date: Date): Date {
  const d = new Date(date);
  const day = d.getDay();
  const diff = d.getDate() - day + (day === 0 ? -6 : 1);
  d.setDate(diff);
  d.setHours(0, 0, 0, 0);
  return d;
}

function formatDate(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

function formatWeekRange(monday: Date): string {
  const sunday = new Date(monday);
  sunday.setDate(sunday.getDate() + 6);
  const opts: Intl.DateTimeFormatOptions = { month: 'short', day: 'numeric' };
  const start = monday.toLocaleDateString('en-US', opts);
  const end = sunday.toLocaleDateString('en-US', { ...opts, year: 'numeric' });
  return `${start} — ${end}`;
}

function getDayOfMonth(dateStr: string): string {
  const d = new Date(dateStr + 'T00:00:00');
  return String(d.getDate()).padStart(2, '0');
}

/**
 * Formats a Date as an ISO 8601 string with the local timezone offset.
 * e.g. "2024-01-15T08:30:00-05:00"
 */
function toISOWithOffset(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  const off = -date.getTimezoneOffset();
  const sign = off >= 0 ? '+' : '-';
  const absOff = Math.abs(off);
  const hh = String(Math.floor(absOff / 60)).padStart(2, '0');
  const mm = String(absOff % 60).padStart(2, '0');
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}` +
    `T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}` +
    `${sign}${hh}:${mm}`
  );
}
