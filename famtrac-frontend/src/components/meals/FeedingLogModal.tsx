import { useState } from 'react';
import { Button } from '../common/Button';
import type { RecipeResponse } from '../../api/types';

interface FeedingLogModalProps {
  show: boolean;
  slotDate: string;
  slotTime: string;
  recipeId?: string;
  recipes: RecipeResponse[];
  onClose: () => void;
  onSubmit: (data: {
    date: string;
    time: string;
    recipe_id: string;
    amount: number;
    reaction: string;
    notes: string;
  }) => void;
  loading: boolean;
}

const AMOUNT_OPTIONS = [
  { value: 0, label: '🤷 Tasted', reaction: 'tasted' },
  { value: 30, label: '😋 Ate some', reaction: 'some' },
  { value: 60, label: '😋 Ate most', reaction: 'most' },
  { value: 90, label: '😋 Ate all', reaction: 'all' },
  { value: 0, label: '🤢 Refused', reaction: 'refused' },
];

/**
 * Modal for logging a feeding from a meal slot.
 */
export function FeedingLogModal({
  show,
  slotDate,
  slotTime,
  recipeId,
  recipes,
  onClose,
  onSubmit,
  loading,
}: FeedingLogModalProps) {
  const [date, setDate] = useState(slotDate);
  const [time, setTime] = useState(slotTime);
  const [selectedRecipeId, setSelectedRecipeId] = useState(recipeId ?? '');
  const [amount, setAmount] = useState(0);
  const [reaction, setReaction] = useState('');
  const [notes, setNotes] = useState('');

  const handleSubmit = () => {
    onSubmit({
      date,
      time,
      recipe_id: selectedRecipeId,
      amount,
      reaction,
      notes,
    });
  };

  if (!show) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="fixed inset-0 bg-black/30" onClick={onClose} />
      <div className="relative z-10 w-full max-w-md bg-white rounded-2xl shadow-xl">
        {/* Header */}
        <div className="flex justify-between items-center p-4 border-b border-gray-100">
          <h3 className="text-base font-semibold">Log Feeding</h3>
          <button onClick={onClose} className="text-gray-400 hover:text-gray-600 p-1">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="p-4 space-y-4">
          {/* Date */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Date</label>
            <input
              type="date"
              value={date}
              onChange={(e) => setDate(e.target.value)}
              className="w-full rounded-xl border border-gray-200 px-3.5 py-2.5 text-sm text-gray-900 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>

          {/* Time */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Time</label>
            <input
              type="time"
              value={time}
              onChange={(e) => setTime(e.target.value)}
              className="w-full rounded-xl border border-gray-200 px-3.5 py-2.5 text-sm text-gray-900 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>

          {/* Recipe */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Recipe</label>
            <select
              value={selectedRecipeId}
              onChange={(e) => setSelectedRecipeId(e.target.value)}
              className="w-full rounded-xl border border-gray-200 px-3.5 py-2.5 text-sm text-gray-900 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="">Select a recipe...</option>
              {recipes.map((r) => (
                <option key={r.id} value={r.id}>
                  {r.emoji || '🍽️'} {r.name}
                </option>
              ))}
            </select>
          </div>

          {/* Amount / Reaction */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              How much did they eat?
            </label>
            <div className="grid grid-cols-1 gap-2">
              {AMOUNT_OPTIONS.map((opt) => (
                <button
                  key={opt.value + opt.reaction}
                  type="button"
                  onClick={() => {
                    setAmount(opt.value);
                    setReaction(opt.reaction);
                  }}
                  className={`text-left px-3 py-2 rounded-xl border text-sm transition-colors ${
                    reaction === opt.reaction
                      ? 'border-blue-500 bg-blue-50 text-blue-700 font-medium'
                      : 'border-gray-200 bg-white hover:bg-gray-50 text-gray-700'
                  }`}
                >
                  {opt.label}
                </button>
              ))}
            </div>
          </div>

          {/* Notes */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Notes (optional)</label>
            <textarea
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              rows={3}
              placeholder="Any notes about this feeding..."
              className="w-full rounded-xl border border-gray-200 px-3.5 py-2.5 text-sm text-gray-900 bg-white placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
            />
          </div>
        </div>

        {/* Footer */}
        <div className="px-4 pb-4 flex gap-2">
          <Button variant="secondary" className="flex-1" onClick={onClose}>
            Cancel
          </Button>
          <Button
            className="flex-1"
            onClick={handleSubmit}
            loading={loading}
            disabled={!date || !time || !selectedRecipeId || !reaction}
          >
            Log Feeding
          </Button>
        </div>
      </div>
    </div>
  );
}
