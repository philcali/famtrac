import { useState, useEffect } from 'react';
import { Button } from '../common/Button';
import type { Recipe, CreateRecipeRequest, UpdateRecipeRequest } from '../../types/domain';

interface RecipeFormProps {
  recipe?: Recipe;
  onSubmit: (data: CreateRecipeRequest | UpdateRecipeRequest) => void;
  onCancel: () => void;
  loading?: boolean;
}

const inputBase =
  'w-full rounded-xl border border-gray-200 px-3.5 py-2.5 text-sm text-gray-900 bg-white placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent';

/**
 * RecipeForm - Modal form for creating or editing a recipe
 * Fields: name, emoji, ingredients, age_min, texture, allergens, prep_notes, safe
 */
export function RecipeForm({ recipe, onSubmit, onCancel, loading = false }: RecipeFormProps) {
  const [name, setName] = useState(recipe?.name ?? '');
  const [emoji, setEmoji] = useState(recipe?.emoji ?? '');
  const [ingredientsText, setIngredientsText] = useState(recipe?.ingredients?.join('\n') ?? '');
  const [ageMin, setAgeMin] = useState(recipe?.age_min?.toString() ?? '');
  const [texture, setTexture] = useState(recipe?.texture ?? '');
  const [allergensText, setAllergensText] = useState(recipe?.allergens?.join('\n') ?? '');
  const [prepNotes, setPrepNotes] = useState(recipe?.prep_notes ?? '');
  const [safe, setSafe] = useState(recipe?.safe ?? false);

  // Sync form fields when recipe prop changes
  /* eslint-disable react-hooks/set-state-in-effect */
  useEffect(() => {
    if (recipe) {
      setName(recipe.name);
      setEmoji(recipe.emoji ?? '');
      setIngredientsText(recipe.ingredients.join('\n'));
      setAgeMin(recipe.age_min?.toString() ?? '');
      setTexture(recipe.texture ?? '');
      setAllergensText(recipe.allergens?.join('\n') ?? '');
      setPrepNotes(recipe.prep_notes ?? '');
      setSafe(recipe.safe ?? false);
    }
  }, [recipe]);
  /* eslint-enable react-hooks/set-state-in-effect */

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const ingredients = ingredientsText
      .split('\n')
      .map((s) => s.trim())
      .filter(Boolean);
    const allergens = allergensText
      .split('\n')
      .map((s) => s.trim())
      .filter(Boolean);

    if (!name.trim()) return;

    const base = {
      name: name.trim(),
      emoji: emoji.trim() || undefined,
      ingredients,
      age_min: ageMin ? Number(ageMin) : undefined,
      texture: texture.trim() || undefined,
      allergens,
      prep_notes: prepNotes.trim() || undefined,
      safe: safe,
    };

    if (recipe) {
      onSubmit(base as UpdateRecipeRequest);
    } else {
      onSubmit(base as CreateRecipeRequest);
    }
  };

  const textureOptions = [
    'smooth',
    'lumpy',
    'chunky',
    'soft',
    'crunchy',
    'mashed',
    'diced',
    'whole',
  ];

  return (
    <form onSubmit={handleSubmit} className="space-y-3">
      {/* Name + Emoji row */}
      <div className="flex gap-2">
        <div className="flex-1">
          <label className="block text-sm font-medium text-gray-700 mb-1">Name *</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g., Banana Oat Pancakes"
            required
            className={inputBase}
          />
        </div>
        <div className="w-16 shrink-0">
          <label className="block text-sm font-medium text-gray-700 mb-1">Emoji</label>
          <input
            type="text"
            value={emoji}
            onChange={(e) => setEmoji(e.target.value)}
            placeholder="🍌"
            maxLength={4}
            className={`${inputBase} text-center text-xl`}
          />
          {emoji && <div className="text-center mt-0.5 text-2xl">{emoji}</div>}
        </div>
      </div>

      {/* Age + Texture row */}
      <div className="flex gap-2">
        <div className="w-28 shrink-0">
          <label className="block text-sm font-medium text-gray-700 mb-1">Min age (mo)</label>
          <input
            type="number"
            value={ageMin}
            onChange={(e) => setAgeMin(e.target.value)}
            placeholder="6"
            min="0"
            max="96"
            className={inputBase}
          />
        </div>
        <div className="flex-1">
          <label className="block text-sm font-medium text-gray-700 mb-1">Texture</label>
          <select
            value={texture}
            onChange={(e) => setTexture(e.target.value)}
            className={inputBase}
          >
            <option value="">Select texture...</option>
            {textureOptions.map((t) => (
              <option key={t} value={t}>
                {t.charAt(0).toUpperCase() + t.slice(1)}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Ingredients */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">
          Ingredients <span className="text-gray-400 font-normal">(one per line)</span>
        </label>
        <textarea
          value={ingredientsText}
          onChange={(e) => setIngredientsText(e.target.value)}
          rows={2}
          className={`${inputBase} resize-none`}
          placeholder={'banana\noats\nmilk'}
        />
      </div>

      {/* Allergens + Prep Notes row */}
      <div className="flex gap-2">
        <div className="flex-1">
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Allergens <span className="text-gray-400 font-normal">(one per line)</span>
          </label>
          <textarea
            value={allergensText}
            onChange={(e) => setAllergensText(e.target.value)}
            rows={2}
            className={`${inputBase} resize-none`}
            placeholder={'milk\ngluten\neggs'}
          />
        </div>
        <div className="flex-1">
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Prep notes <span className="text-gray-400 font-normal">(optional)</span>
          </label>
          <textarea
            value={prepNotes}
            onChange={(e) => setPrepNotes(e.target.value)}
            rows={2}
            className={`${inputBase} resize-none`}
            placeholder="Cooking tips, storage instructions, etc."
          />
        </div>
      </div>

      {/* Safe toggle — compact inline pill */}
      <div className="flex items-center gap-2 pt-1">
        <label
          className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium cursor-pointer select-none transition-colors ${
            safe ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-500 hover:bg-gray-200'
          }`}
        >
          <input
            type="checkbox"
            checked={safe}
            onChange={(e) => setSafe(e.target.checked)}
            className="w-3 h-3 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
          />
          <span>Mark as safe</span>
        </label>
      </div>

      {/* Actions */}
      <div className="flex gap-2 pt-1">
        <Button variant="primary" type="submit" loading={loading} disabled={loading}>
          {recipe ? 'Save Changes' : 'Add Recipe'}
        </Button>
        <Button variant="secondary" onClick={onCancel} disabled={loading}>
          Cancel
        </Button>
      </div>
    </form>
  );
}
