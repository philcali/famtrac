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
import type { RecipeResponse } from '../api/types';
import type { MealSlot, CreateMealSlotRequest, UpdateMealSlotRequest } from '../types/domain';
import type { CreateFeedingLogRequest } from '../types/domain';

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
  const {
    mutate: createSlotMutation,
  } = useApiMutation(
    (data: CreateMealSlotRequest) => createMealSlot(apiClient, familyId ?? 'NA', dependentId ?? 'NA', data)
  );

  const {
    mutate: updateSlotMutation,
  } = useApiMutation(
    (data: UpdateMealSlotRequest) =>
      updateMealSlot(apiClient, familyId ?? 'NA', dependentId ?? 'NA', editingSlot!.id, data)
  );

  const {
    mutate: deleteSlotMutation,
    loading: deleteSlotLoading,
  } = useApiMutation((id: string) => deleteMealSlot(apiClient, familyId ?? 'NA', dependentId ?? 'NA', id));

  const {
    mutate: createFeedingLogMutation,
    loading: feedingLogLoading,
  } = useApiMutation(
    (data: CreateFeedingLogRequest) =>
      createFeedingLog(apiClient, familyId ?? 'NA', dependentId ?? 'NA', data)
  );

  // ---- Derived data ----
  const recipes: RecipeResponse[] = recipesData?.recipes ?? [];
  const slots = slotsData?.meal_slots ?? [];

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
      const response = await updateSlotMutation({ recipe_id: recipe.id });
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
    const response = await createFeedingLogMutation({
      family_id: familyId ?? 'NA',
      dependent_id: dependentId ?? 'NA',
      date: data.date,
      time: data.time,
      recipe_id: data.recipe_id,
      amount: data.amount,
      reaction: data.reaction,
      notes: data.notes,
    });
    if (!response.error) {
      setSuccessMessage('Feeding logged');
      setFeedingSlot(undefined);
      refetchSlots();
    }
  };

  const handleDeleteSlot = () => {
    if (!deletingSlot) return;
    deleteSlotMutation(deletingSlot.id);
    setSuccessMessage('Meal removed from plan');
    setDeletingSlot(undefined);
    refetchSlots();
  };

  const handleBackClick = () => navigate(-1);
  const handleSuccessClose = useCallback(() => setSuccessMessage(null), []);

  // ---- Loading state ----
  if (recipesLoading || slotsLoading) {
    return (
      <div className="py-4 max-w-5xl mx-auto px-4">
        <div className="mb-4">
          <h2 className="heading">
            Meal Plan
            <Button variant="secondary" onClick={handleBackClick} className="heading-right">
              ← Back
            </Button>
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
            <Button variant="secondary" onClick={handleBackClick} className="heading-right">
              ← Back
            </Button>
          </h2>
        </div>
        <ErrorMessage message={recipesError ?? slotsError ?? 'An error occurred'} />
        <Button onClick={handleBackClick} className="mt-3">
          ← Back
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
          <Button variant="secondary" onClick={handleBackClick} className="heading-right">
            ← Back
          </Button>
        </h2>
      </div>

      {/* Week navigation */}
      <div className="flex items-center justify-between mb-4">
        <Button variant="secondary" size="sm" onClick={goToPrevWeek}>
          ← Prev
        </Button>
        <div className="text-sm text-gray-600">
          {formatWeekRange(currentWeekStart)}
        </div>
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
                  className={`text-[10px] mt-0.5 ${
                    isSelected ? 'text-blue-100' : 'text-gray-400'
                  }`}
                >
                  {day.count}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {/* Success message */}
      {successMessage && (
        <SuccessMessage message={successMessage} onClose={handleSuccessClose} />
      )}

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
            <p className="text-sm text-gray-500 mt-1">
              Add a recipe to get started.
            </p>
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
          <Button variant="outline-secondary" className="w-full" icon="plus" onClick={handleAddSlot}>
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
          loading={feedingLogLoading}
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
