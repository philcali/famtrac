import { useState, useMemo } from 'react';
import { Badge } from '../common/Badge';
import type { RecipeResponse } from '../../api/types';

interface RecipePickerModalProps {
  show: boolean;
  recipes: RecipeResponse[];
  onSelect: (recipe: RecipeResponse) => void;
  onClose: () => void;
}

/**
 * Modal that lets the user search and select a recipe from the library.
 */
export function RecipePickerModal({ show, recipes, onSelect, onClose }: RecipePickerModalProps) {
  const [search, setSearch] = useState('');

  const filtered = useMemo(() => {
    if (!search.trim()) return recipes;
    const q = search.toLowerCase();
    return recipes.filter(
      (r) =>
        r.name.toLowerCase().includes(q) || r.ingredients.some((i) => i.toLowerCase().includes(q))
    );
  }, [recipes, search]);

  if (!show) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="fixed inset-0 bg-black/30" onClick={onClose} />
      <div className="relative z-10 w-full max-w-2xl bg-white rounded-2xl shadow-xl max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex justify-between items-center p-4 border-b border-gray-100">
          <h3 className="text-base font-semibold">Add Recipe</h3>
          <button onClick={onClose} className="text-gray-400 hover:text-gray-600 p-1">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* Search */}
        <div className="px-4 pt-4">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search recipes by name or ingredient..."
            className="w-full rounded-xl border border-gray-200 px-3.5 py-2.5 text-sm text-gray-900 bg-white placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>

        {/* Recipe list */}
        <div className="flex-1 overflow-y-auto px-4 py-3">
          {filtered.length === 0 ? (
            <div className="text-center py-8">
              <span className="text-3xl">🔍</span>
              <p className="text-sm text-gray-500 mt-2">No matching recipes</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {filtered.map((recipe) => (
                <button
                  key={recipe.id}
                  onClick={() => onSelect(recipe)}
                  className="text-left bg-gray-50 rounded-xl border border-gray-100 p-3 hover:bg-blue-50 hover:border-blue-200 transition-colors"
                >
                  <div className="flex items-start gap-2">
                    <span className="text-2xl flex-shrink-0">{recipe.emoji || '🍽️'}</span>
                    <div className="min-w-0">
                      <p className="text-sm font-semibold text-gray-900 truncate">{recipe.name}</p>
                      <div className="flex flex-wrap gap-1 mt-1">
                        {recipe.age_min != null && (
                          <Badge variant="secondary" className="text-[10px] px-1.5 py-0">
                            {recipe.age_min}+m
                          </Badge>
                        )}
                        {recipe.texture && (
                          <Badge variant="info" className="text-[10px] px-1.5 py-0">
                            {recipe.texture}
                          </Badge>
                        )}
                        {recipe.allergens && recipe.allergens.length > 0 && (
                          <Badge variant="danger" className="text-[10px] px-1.5 py-0">
                            ⚠️ {recipe.allergens.length}
                          </Badge>
                        )}
                      </div>
                    </div>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
