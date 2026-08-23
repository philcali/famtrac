import { Badge } from '../common/Badge';
import { Button } from '../common/Button';
import type { RecipeResponse } from '../../api/types';

interface RecipeCardProps {
  recipe: RecipeResponse;
  onEdit: (recipe: RecipeResponse) => void;
  onDelete: (recipe: RecipeResponse) => void;
}

/**
 * RecipeCard - Displays a single recipe with emoji, name, age, texture, and allergen badges
 */
export function RecipeCard({ recipe, onEdit, onDelete }: RecipeCardProps) {
  const textureVariant: Record<string, string> = {
    smooth: 'primary',
    lumpy: 'info',
    chunky: 'warning',
    soft: 'success',
    crunchy: 'danger',
  };

  const allergenVariant: string = 'danger';

  return (
    <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-4 hover:shadow-md transition-shadow">
      <div className="flex items-start gap-3">
        {/* Emoji */}
        <span className="text-3xl flex-shrink-0">{recipe.emoji || '🍽️'}</span>

        {/* Name + Badges */}
        <div className="flex-1 min-w-0">
          <h3 className="text-base font-semibold text-gray-900 truncate">{recipe.name}</h3>

          {/* Age + Texture row */}
          <div className="flex flex-wrap gap-1.5 mt-1.5">
            {recipe.age_min != null && <Badge variant="secondary">{recipe.age_min}+ months</Badge>}
            {recipe.texture && (
              <Badge variant={textureVariant[recipe.texture.toLowerCase()] || 'secondary'}>
                {recipe.texture}
              </Badge>
            )}
          </div>

          {/* Allergens */}
          {recipe.allergens.length > 0 && (
            <div className="flex flex-wrap gap-1.5 mt-1.5">
              {recipe.allergens.map((a) => (
                <Badge key={a} variant={allergenVariant}>
                  ⚠️ {a}
                </Badge>
              ))}
            </div>
          )}

          {/* Safe badge */}
          {recipe.safe && (
            <Badge variant="success" className="mt-1.5">
              ✓ Safe
            </Badge>
          )}
        </div>

        {/* Actions */}
        <div className="flex gap-1.5 flex-shrink-0">
          <Button variant="secondary" size="sm" onClick={() => onEdit(recipe)}>
            Edit
          </Button>
          <Button variant="outline-danger" size="sm" onClick={() => onDelete(recipe)}>
            Delete
          </Button>
        </div>
      </div>

      {/* Prep notes */}
      {recipe.prep_notes && (
        <p className="mt-3 text-sm text-gray-500 border-t border-gray-100 pt-3">
          {recipe.prep_notes}
        </p>
      )}
    </div>
  );
}
