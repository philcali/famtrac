import { Badge } from '../common/Badge';
import { Button } from '../common/Button';

export interface MealSlotCardProps {
  slotId: string;
  time: string;
  recipeEmoji?: string;
  recipeName?: string;
  recipeAgeMin?: number;
  recipeTexture?: string;
  recipeAllergens?: string[];
  recipeSafe?: boolean;
  notes?: string;
  onEdit: (slotId: string) => void;
  onLogFeeding: (slotId: string) => void;
  onDelete: (slotId: string) => void;
}

const TEXTURE_VARIANT: Record<string, string> = {
  smooth: 'primary',
  lumpy: 'info',
  chunky: 'warning',
  soft: 'success',
  crunchy: 'danger',
};

/**
 * Displays a single meal slot card with recipe info, time, and actions.
 */
export function MealSlotCard({
  slotId,
  time,
  recipeEmoji,
  recipeName,
  recipeAgeMin,
  recipeTexture,
  recipeAllergens,
  recipeSafe,
  notes,
  onEdit,
  onLogFeeding,
  onDelete,
}: MealSlotCardProps) {
  return (
    <div className="bg-white rounded-xl border border-gray-100 shadow-sm p-4 hover:shadow-md transition-shadow">
      <div className="flex items-start gap-3">
        {/* Time + Emoji */}
        <div className="flex flex-col items-center gap-1 flex-shrink-0">
          <span className="text-sm font-semibold text-gray-500">{time}</span>
          <span className="text-2xl">{recipeEmoji || '🍽️'}</span>
        </div>

        {/* Name + Badges */}
        <div className="flex-1 min-w-0">
          <h3 className="text-base font-semibold text-gray-900 truncate">
            {recipeName || 'No recipe selected'}
          </h3>

          {/* Age + Texture row */}
          <div className="flex flex-wrap gap-1.5 mt-1.5">
            {recipeAgeMin != null && (
              <Badge variant="secondary">{recipeAgeMin}+ months</Badge>
            )}
            {recipeTexture && (
              <Badge variant={TEXTURE_VARIANT[recipeTexture.toLowerCase()] || 'secondary'}>
                {recipeTexture}
              </Badge>
            )}
          </div>

          {/* Allergens */}
          {recipeAllergens && recipeAllergens.length > 0 && (
            <div className="flex flex-wrap gap-1.5 mt-1.5">
              {recipeAllergens.map((a) => (
                <Badge key={a} variant="danger">
                  ⚠️ {a}
                </Badge>
              ))}
            </div>
          )}

          {/* Safe badge */}
          {recipeSafe && (
            <Badge variant="success" className="mt-1.5">
              ✓ Safe
            </Badge>
          )}

          {/* Notes */}
          {notes && (
            <p className="mt-2 text-sm text-gray-500 border-t border-gray-100 pt-2">
              {notes}
            </p>
          )}
        </div>

        {/* Actions */}
        <div className="flex gap-1.5 flex-shrink-0">
          <Button variant="secondary" size="sm" onClick={() => onEdit(slotId)}>
            Edit
          </Button>
          <Button variant="secondary" size="sm" onClick={() => onLogFeeding(slotId)}>
            Log
          </Button>
          <Button variant="outline-danger" size="sm" onClick={() => onDelete(slotId)}>
            Delete
          </Button>
        </div>
      </div>
    </div>
  );
}
