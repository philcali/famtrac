import { useState, useCallback, useEffect } from 'react';
import { Button } from '../common/Button';
import type {
  LittleEaterExport,
  LittleEaterRecipe,
  CreateRecipeRequest,
  LittleEaterFeedingLog,
} from '../../types/domain';

/**
 * Map Little Eater reaction values to famtrac reaction values.
 * Handles both export format ("ate_some") and backup format ("Ate some").
 */
const mapReaction = (littleEaterReaction?: string): string => {
  const normalized = (littleEaterReaction ?? '').toLowerCase().replace(/\s+/g, '_');
  switch (normalized) {
    case 'ate_some':
      return 'some';
    case 'ate_most':
      return 'most';
    case 'ate_all':
      return 'all';
    case 'none':
      return 'none';
    default:
      return littleEaterReaction ?? 'some';
  }
};

/**
 * Map Little Eater texture values to famtrac texture values.
 * Little Eater uses different texture vocabulary (e.g., "pureed", "mushy", "thick")
 * while famtrac uses: smooth, lumpy, chunky, soft, crunchy, mashed, diced, whole.
 */
const LITTLE_EATER_TO_FAMTRAC_TEXTURE: Record<string, string> = {
  pureed: 'smooth',
  smooth: 'smooth',
  thick_puree: 'smooth',
  mushy: 'mashed',
  mashed: 'mashed',
  soft_mashed: 'mashed',
  lumpy: 'lumpy',
  thick: 'chunky',
  chunky: 'chunky',
  soft_chunky: 'chunky',
  soft: 'soft',
  soft_cooked: 'soft',
  crunchy: 'crunchy',
  hard: 'crunchy',
  diced: 'diced',
  chopped: 'diced',
  small_cubes: 'diced',
  whole: 'whole',
  finger_food: 'whole',
  soft_finger_food: 'whole',
  // localStorage backup format values
  thinned: 'smooth',
  puree: 'smooth',
};

/**
 * localStorage backup format from food-plan (Little Eater) app.
 * Different from the food-plan export format:
 * - No `type` field
 * - Data nested under `localStorage.fp_*` keys
 * - camelCase field names (ingredient, ageMin, prepNotes)
 * - `ingredient` is a single string (not an array)
 */
interface LocalStorageBackup {
  version: number;
  exportedAt?: string;
  localStorage: {
    fp_recipes: LocalStorageRecipe[];
    fp_feeding_logs: LocalStorageFeedingLog[];
  };
}

interface LocalStorageRecipe {
  name: string;
  emoji?: string;
  ingredient: string;
  ageMin?: number;
  texture?: string;
  allergens: string[];
  prepNotes?: string;
  safe?: boolean;
  ingredients: string[];
}

interface LocalStorageFeedingLog {
  date: string;
  time: string;
  dependent_name: string;
  recipe_name: string;
  ingredient: string;
  amount: string;
  reaction: string;
  notes?: string;
  dependent_id: string;
}

type ImportStep = 'preview' | 'importRecipes' | 'importFeedingLogs' | 'results';

interface ImportModalProps {
  familyId: string;
  onClose: () => void;
  onRecipesImported: (count: number) => void;
  onFeedingLogsImported: (count: number) => void;
}

interface DependentOption {
  id: string;
  name: string;
}

/**
 * ImportModal - Import recipes and feeding logs from a Little Eater export JSON file.
 *
 * Flow:
 * 1. File picker → parse & validate
 * 2. Preview (recipe count, feeding log count)
 * 3. Import recipes (auto-proceeds)
 * 4. Import feeding logs (requires dependent selection)
 * 5. Results summary
 */
export function ImportModal({
  familyId,
  onClose,
  onRecipesImported,
  onFeedingLogsImported,
}: ImportModalProps) {
  const [step, setStep] = useState<ImportStep>('preview');
  const [exportData, setExportData] = useState<LittleEaterExport | null>(null);
  const [importError, setImportError] = useState<string | null>(null);
  const [recipeImportResult, setRecipeImportResult] = useState<{
    success: number;
    failed: number;
  } | null>(null);
  const [feedingLogImportResult, setFeedingLogImportResult] = useState<{
    success: number;
    failed: number;
  } | null>(null);
  const [selectedDependentId, setSelectedDependentId] = useState<string>('');
  const [dependents, setDependents] = useState<DependentOption[]>([]);
  const [importing, setImporting] = useState(false);

  // Fetch dependents for the family
  useEffect(() => {
    let cancelled = false;
    fetch(`/families/${familyId}/dependents`)
      .then((res) => res.json())
      .then((data: { dependents: { id: string; name: string }[] }) => {
        if (!cancelled) {
          setDependents(data.dependents);
        }
      })
      .catch(() => {
        if (!cancelled) setDependents([]);
      });
    return () => {
      cancelled = true;
    };
  }, [familyId]);

  const handleFileChange = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setImportError(null);

    try {
      const text = await file.text();
      const raw = JSON.parse(text);

      // Detect localStorage backup format (has `localStorage` key, no `type` key)
      let exportData: LittleEaterExport;

      if ('localStorage' in raw && 'fp_recipes' in raw.localStorage) {
        // localStorage backup format — normalize to expected structure
        const backup: LocalStorageBackup = raw as unknown as LocalStorageBackup;

        exportData = {
          version: backup.version,
          type: 'full' as const,
          recipes: backup.localStorage.fp_recipes as LittleEaterRecipe[],
          feeding_logs: backup.localStorage.fp_feeding_logs.map((log) => ({
            date: log.date,
            time: log.time,
            dependent_name: log.dependent_name,
            recipe_name: log.recipe_name,
            // Backup format uses `amount` (string like "Ate some") and `reaction` ("None")
            // Standard format uses `reaction` (string like "ate_some")
            dependent_id: log.dependent_id,
            amount: 30,
            reaction: log.amount ? mapReaction(log.amount) : mapReaction(log.reaction),
            notes: log.notes,
          })) as LittleEaterFeedingLog[],
        };
      } else if (
        'type' in raw &&
        (raw.type === 'recipes' || raw.type === 'full') &&
        Array.isArray(raw.recipes)
      ) {
        // Standard food-plan export format
        exportData = raw as LittleEaterExport;

        if (exportData.type === 'full' && !Array.isArray(exportData.feeding_logs)) {
          setImportError('Export type is "full" but missing "feeding_logs" array.');
          e.target.value = '';
          return;
        }
      } else {
        setImportError(
          'Unsupported file format. Expected a Little Eater export or localStorage backup.'
        );
        e.target.value = '';
        return;
      }

      setExportData(exportData);
      setStep('preview');
    } catch (err: unknown) {
      const message =
        err instanceof Error
          ? err.message
          : 'Failed to parse JSON file. Please ensure it is a valid Little Eater export.';
      setImportError(message);
    }

    // Reset file input so re-selecting the same file triggers onChange
    e.target.value = '';
  }, []);

  const mapTexture = useCallback((littleEaterTexture?: string): string | undefined => {
    if (!littleEaterTexture) return undefined;
    const key = littleEaterTexture.toLowerCase().replace(/\s+/g, '_');
    return LITTLE_EATER_TO_FAMTRAC_TEXTURE[key] ?? littleEaterTexture;
  }, []);

  const mapRecipeToCreateRequest = useCallback(
    (r: LittleEaterRecipe | LocalStorageRecipe): CreateRecipeRequest => ({
      name: r.name,
      emoji: r.emoji,
      // Export format uses `ingredients` (array), backup format uses `ingredient` (string)
      ingredients: Array.isArray((r as LittleEaterRecipe).ingredients)
        ? (r as LittleEaterRecipe).ingredients
        : [(r as LocalStorageRecipe).ingredient],
      // Export format uses `age_min`, backup format uses `ageMin`
      age_min: (r as LittleEaterRecipe).age_min ?? (r as LocalStorageRecipe).ageMin,
      texture: mapTexture(r.texture),
      // Export format uses `allergens`, backup format uses `allergens` (same)
      allergens: (r as LittleEaterRecipe).allergens ?? (r as LocalStorageRecipe).allergens,
      // Export format uses `prep_notes`, backup format uses `prepNotes`
      prep_notes: (r as LittleEaterRecipe).prep_notes ?? (r as LocalStorageRecipe).prepNotes,
      safe: r.safe,
    }),
    [mapTexture]
  );

  const reactionToVolume = useCallback((reaction?: string): number => {
    switch (mapReaction(reaction)) {
      case 'tasted':
        return 10;
      case 'some':
        return 30;
      case 'most':
        return 60;
      case 'all':
        return 90;
      case 'refused':
        return 0;
      case 'none':
        return 0;
      default:
        return 30;
    }
  }, []);

  const handleImportRecipes = useCallback(async () => {
    if (!exportData) return;

    setImportError(null);
    setImporting(true);

    let success = 0;
    let failed = 0;

    for (const recipe of exportData.recipes) {
      try {
        const response = await fetch(`/families/${familyId}/recipes`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(mapRecipeToCreateRequest(recipe)),
        });

        if (response.ok) {
          success++;
        } else {
          failed++;
        }
      } catch {
        failed++;
      }
    }

    setImporting(false);
    setRecipeImportResult({ success, failed });

    // Auto-proceed to feeding logs if present
    if (exportData.feeding_logs.length > 0) {
      setStep('importFeedingLogs');
    } else {
      setStep('results');
    }
  }, [exportData, familyId, mapRecipeToCreateRequest]);

  const handleImportFeedingLogs = useCallback(async () => {
    if (!exportData || !selectedDependentId) return;

    setImportError(null);
    setImporting(true);

    let success = 0;
    let failed = 0;

    for (const log of exportData.feeding_logs) {
      try {
        const response = await fetch(
          `/families/${familyId}/dependents/${selectedDependentId}/activities`,
          {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              family_id: familyId,
              dependent_id: selectedDependentId,
              type: 'feeding',
              timestamp: `${log.date}T${log.time}:00`,
              feeding_type: 'solid',
              volume_ml: reactionToVolume(log.reaction),
              notes: log.notes,
            }),
          }
        );

        if (response.ok) {
          success++;
        } else {
          failed++;
        }
      } catch {
        failed++;
      }
    }

    setImporting(false);
    setFeedingLogImportResult({ success, failed });
    setStep('results');
  }, [exportData, familyId, reactionToVolume, selectedDependentId]);

  const handleDone = useCallback(() => {
    onRecipesImported(recipeImportResult?.success ?? 0);
    onFeedingLogsImported(feedingLogImportResult?.success ?? 0);
    onClose();
  }, [
    onClose,
    onRecipesImported,
    onFeedingLogsImported,
    recipeImportResult,
    feedingLogImportResult,
  ]);

  const hasFeedingLogs = exportData !== null && exportData.feeding_logs.length > 0;

  // Modal rendering
  const renderModal = (title: string, body: React.ReactNode, footer?: React.ReactNode) => (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="fixed inset-0 bg-black/30" onClick={onClose} />
      <div className="relative z-10 w-full max-w-md bg-white rounded-2xl shadow-xl">
        <div className="flex justify-between items-center p-4 border-b border-gray-100">
          <h3 className="text-base font-semibold">{title}</h3>
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
        <div className="p-4">{body}</div>
        {footer && <div className="px-4 pb-4">{footer}</div>}
      </div>
    </div>
  );

  // Step 1: Preview
  const renderPreview = () => (
    <>
      <p className="text-sm text-gray-600 mb-4">Preview of your Little Eater export:</p>

      <div className="space-y-3 mb-4">
        <div className="flex items-center justify-between p-3 bg-blue-50 rounded-xl">
          <div>
            <p className="text-sm font-medium text-gray-900">Recipes</p>
            <p className="text-xs text-gray-500">Will be imported automatically</p>
          </div>
          <span className="text-lg font-semibold text-blue-600">
            {exportData?.recipes.length ?? 0}
          </span>
        </div>

        {hasFeedingLogs && (
          <div className="flex items-center justify-between p-3 bg-green-50 rounded-xl">
            <div>
              <p className="text-sm font-medium text-gray-900">Feeding Logs</p>
              <p className="text-xs text-gray-500">Will be imported as feeding activities</p>
            </div>
            <span className="text-lg font-semibold text-green-600">
              {exportData?.feeding_logs.length ?? 0}
            </span>
          </div>
        )}
      </div>

      {hasFeedingLogs && (
        <div className="mb-4">
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Import feeding logs to dependent:
          </label>
          <select
            value={selectedDependentId}
            onChange={(e) => setSelectedDependentId(e.target.value)}
            className="w-full rounded-xl border border-gray-200 px-3.5 py-2.5 text-sm text-gray-900 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="">Select a dependent...</option>
            {dependents.map((dep) => (
              <option key={dep.id} value={dep.id}>
                {dep.name}
              </option>
            ))}
          </select>
          {dependents.length === 0 && (
            <p className="text-xs text-amber-600 mt-1">
              No dependents found. Create a dependent first to import feeding logs.
            </p>
          )}
        </div>
      )}

      {importError && (
        <div className="mb-4 p-3 bg-red-50 border border-red-100 rounded-xl text-red-700 text-sm">
          {importError}
        </div>
      )}

      {step === 'preview' && (
        <div className="mt-4">
          <label className="block text-sm font-medium text-gray-700 mb-1">Change file</label>
          <input
            type="file"
            accept=".json,application/json"
            onChange={handleFileChange}
            className="w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-xl file:border-0 file:text-sm file:font-semibold file:bg-blue-50 file:text-blue-700 hover:file:bg-blue-100 cursor-pointer"
          />
        </div>
      )}
    </>
  );

  // Step 2: Import Recipes
  const renderImportRecipes = () => (
    <>
      <div className="text-center py-4">
        <span className="text-4xl">🍳</span>
        <p className="text-sm text-gray-600 mt-2">
          Importing {exportData?.recipes.length ?? 0} recipes...
        </p>
      </div>
      {importError && (
        <div className="mb-4 p-3 bg-red-50 border border-red-100 rounded-xl text-red-700 text-sm">
          {importError}
        </div>
      )}
    </>
  );

  // Step 3: Import Feeding Logs
  const renderImportFeedingLogs = () => (
    <>
      <p className="text-sm text-gray-600 mb-4">
        Importing feeding logs to <strong>{exportData?.feeding_logs.length ?? 0} logs</strong>
        ...
      </p>
      {importError && (
        <div className="mb-4 p-3 bg-red-50 border border-red-100 rounded-xl text-red-700 text-sm">
          {importError}
        </div>
      )}
    </>
  );

  // Step 4: Results
  const renderResults = () => {
    const totalRecipes = (recipeImportResult?.success ?? 0) + (recipeImportResult?.failed ?? 0);
    const totalFeedingLogs =
      (feedingLogImportResult?.success ?? 0) + (feedingLogImportResult?.failed ?? 0);

    return (
      <>
        <div className="space-y-4">
          <div className="p-3 bg-green-50 border border-green-100 rounded-xl">
            <p className="text-sm font-medium text-green-800">Recipes imported</p>
            <p className="text-2xl font-semibold text-green-600">
              {recipeImportResult?.success ?? 0}
              <span className="text-sm text-green-500 ml-1">/ {totalRecipes}</span>
            </p>
            {recipeImportResult?.failed ? (
              <p className="text-xs text-green-600 mt-1">
                {recipeImportResult.failed} failed (may already exist)
              </p>
            ) : null}
          </div>

          {feedingLogImportResult && (
            <div className="p-3 bg-blue-50 border border-blue-100 rounded-xl">
              <p className="text-sm font-medium text-blue-800">Feeding logs imported</p>
              <p className="text-2xl font-semibold text-blue-600">
                {feedingLogImportResult.success}
                <span className="text-sm text-blue-500 ml-1">/ {totalFeedingLogs}</span>
              </p>
              {feedingLogImportResult.failed ? (
                <p className="text-xs text-blue-600 mt-1">{feedingLogImportResult.failed} failed</p>
              ) : null}
            </div>
          )}
        </div>
      </>
    );
  };

  return renderModal(
    'Import from Little Eater',
    <>
      {step === 'preview' && renderPreview()}
      {step === 'importRecipes' && renderImportRecipes()}
      {step === 'importFeedingLogs' && renderImportFeedingLogs()}
      {step === 'results' && renderResults()}
    </>,
    <>
      {step === 'preview' && (
        <div className="flex gap-2">
          <Button variant="secondary" onClick={onClose} disabled={importing}>
            Cancel
          </Button>
          <Button
            onClick={handleImportRecipes}
            disabled={importing || (hasFeedingLogs && !selectedDependentId)}
          >
            {importing ? 'Importing...' : 'Import All'}
          </Button>
        </div>
      )}
      {step === 'importRecipes' && (
        <div className="flex gap-2">
          <Button variant="secondary" onClick={onClose} disabled={importing}>
            Cancel
          </Button>
          <Button disabled={true}>{importing ? 'Importing...' : 'Importing'}</Button>
        </div>
      )}
      {step === 'importFeedingLogs' && (
        <div className="flex gap-2">
          <Button variant="secondary" onClick={onClose} disabled={importing}>
            Cancel
          </Button>
          <Button onClick={handleImportFeedingLogs} disabled={importing || !selectedDependentId}>
            {importing ? 'Importing...' : 'Import Feeding Logs'}
          </Button>
        </div>
      )}
      {step === 'results' && <Button onClick={handleDone}>Done</Button>}
    </>
  );
}
