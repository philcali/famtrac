import { useState, useMemo, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Button } from '../components/common/Button';
import { ConfirmDialog } from '../components/common/ConfirmDialog';
import { SuccessMessage } from '../components/common/SuccessMessage';
import { ErrorMessage } from '../components/common/ErrorMessage';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { SkeletonCard } from '../components/common/SkeletonCard';
import { RecipeCard } from '../components/recipes/RecipeCard';
import { RecipeForm } from '../components/recipes/RecipeForm';
import { useAuth } from '../auth/useAuth';
import { useApi, useApiMutation } from '../hooks/useApi';
import { createApiClient } from '../api/client';
import { getRecipes, createRecipe, updateRecipe, deleteRecipe } from '../api/recipes';
import type { Recipe, CreateRecipeRequest, UpdateRecipeRequest } from '../types/domain';

/**
 * RecipeLibraryPage - Browse and manage the family's recipe collection
 * - Lists all recipes with emoji, name, age, texture, allergen badges
 * - Search bar filters by name and ingredients
 * - "Add Recipe" modal form
 * - Edit and delete actions on each recipe card
 */
export function RecipeLibraryPage() {
  const { familyId } = useParams<{ familyId: string }>();
  const navigate = useNavigate();
  const { getToken } = useAuth();

  const [searchQuery, setSearchQuery] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [editingRecipe, setEditingRecipe] = useState<Recipe | undefined>();
  const [deletingRecipe, setDeletingRecipe] = useState<Recipe | undefined>();
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const apiClient = createApiClient(getToken);

  // Fetch recipes for this family
  const {
    data: recipesData,
    loading: recipesLoading,
    error: recipesError,
    refetch: refetchRecipes,
  } = useApi(() => getRecipes(apiClient, familyId ?? 'NA'), [familyId]);

  const recipes = recipesData?.recipes ?? [];

  // Create recipe mutation
  const { mutate: createRecipeMutation, loading: createLoading } = useApiMutation(
    (data: CreateRecipeRequest) => createRecipe(apiClient, familyId ?? 'NA', data)
  );

  // Update recipe mutation
  const { mutate: updateRecipeMutation, loading: updateLoading } = useApiMutation(
    (data: UpdateRecipeRequest) => updateRecipe(apiClient, familyId ?? 'NA', editingRecipe!.id, data)
  );

  // Delete recipe mutation
  const { mutate: deleteRecipeMutation, loading: deleteLoading } = useApiMutation(
    (id: string) => deleteRecipe(apiClient, familyId ?? 'NA', id)
  );

  // Filter recipes by search query (name + ingredients)
  const filteredRecipes = useMemo(() => {
    if (!searchQuery.trim()) return recipes;
    const q = searchQuery.toLowerCase();
    return recipes.filter(
      (r) =>
        r.name.toLowerCase().includes(q) ||
        r.ingredients.some((i) => i.toLowerCase().includes(q))
    );
  }, [recipes, searchQuery]);

  // Handlers
  const handleAddClick = () => {
    setEditingRecipe(undefined);
    setShowForm(true);
  };

  const handleEditClick = (recipe: Recipe) => {
    setEditingRecipe(recipe);
    setShowForm(true);
  };

  const handleDeleteClick = (recipe: Recipe) => {
    setDeletingRecipe(recipe);
  };

  const handleFormSubmit = async (data: CreateRecipeRequest | UpdateRecipeRequest) => {
    if (editingRecipe) {
      const response = await updateRecipeMutation(data as UpdateRecipeRequest);
      if (!response.error) {
        setSuccessMessage('Recipe updated successfully');
        setShowForm(false);
        setEditingRecipe(undefined);
        refetchRecipes();
      }
    } else {
      const response = await createRecipeMutation(data as CreateRecipeRequest);
      if (!response.error) {
        setSuccessMessage('Recipe added successfully');
        setShowForm(false);
        refetchRecipes();
      }
    }
  };

  const handleFormCancel = () => {
    setShowForm(false);
    setEditingRecipe(undefined);
  };

  const handleDeleteConfirm = async () => {
    if (deletingRecipe) {
      const response = await deleteRecipeMutation(deletingRecipe.id);
      if (!response.error) {
        setSuccessMessage('Recipe deleted successfully');
        setDeletingRecipe(undefined);
        refetchRecipes();
      }
    }
  };

  const handleDeleteCancel = () => {
    setDeletingRecipe(undefined);
  };

  const handleSuccessClose = useCallback(() => {
    setSuccessMessage(null);
  }, []);

  const handleBackClick = () => {
    navigate(-1);
  };

  // Modal renderer (same pattern as FamilyDetailPage)
  const renderModal = (
    title: string,
    body: React.ReactNode,
    onClose: () => void,
    footer?: React.ReactNode
  ) => (
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

  // Loading state
  if (recipesLoading) {
    return (
      <div className="py-4 max-w-5xl mx-auto px-4">
        <div className="mb-4">
          <h2 className="heading">
            Recipes
            <Button variant="secondary" onClick={handleBackClick} className="heading-right">
              ← Back
            </Button>
          </h2>
        </div>
        <SkeletonCard count={4} />
      </div>
    );
  }

  // Error state
  if (recipesError) {
    return (
      <div className="py-4 max-w-5xl mx-auto px-4">
        <div className="mb-4">
          <h2 className="heading">
            Recipes
            <Button variant="secondary" onClick={handleBackClick} className="heading-right">
              ← Back
            </Button>
          </h2>
        </div>
        <ErrorMessage message={recipesError} />
        <Button onClick={handleBackClick} className="mt-3">
          ← Back to Families
        </Button>
      </div>
    );
  }

  return (
    <div className="py-4 max-w-5xl mx-auto px-4">
      {/* Header */}
      <div className="mb-4">
        <h2 className="heading">
          Recipes
          <Button variant="secondary" onClick={handleBackClick} className="heading-right">
            ← Back
          </Button>
        </h2>
      </div>

      {/* Search + Add */}
      <div className="flex flex-col sm:flex-row gap-3 mb-4">
        <div className="flex-1">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search recipes by name or ingredient..."
            className="w-full rounded-xl border border-gray-200 px-3.5 py-2.5 text-sm text-gray-900 bg-white placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>
        <Button icon="plus" onClick={handleAddClick}>
          Add Recipe
        </Button>
      </div>

      {/* Success message */}
      {successMessage && (
        <SuccessMessage message={successMessage} onClose={handleSuccessClose} />
      )}

      {/* Recipe grid */}
      {filteredRecipes.length === 0 && recipes.length === 0 ? (
        <div className="text-center py-12 bg-white rounded-xl border border-gray-100">
          <span className="text-4xl">🍽️</span>
          <h3 className="text-lg font-semibold text-gray-900 mt-3">No recipes yet</h3>
          <p className="text-sm text-gray-500 mt-1">
            Start building your family's recipe collection.
          </p>
          <Button className="mt-4" icon="plus" onClick={handleAddClick}>
            Add Your First Recipe
          </Button>
        </div>
      ) : filteredRecipes.length === 0 ? (
        <div className="text-center py-12 bg-white rounded-xl border border-gray-100">
          <span className="text-4xl">🔍</span>
          <h3 className="text-lg font-semibold text-gray-900 mt-3">No matching recipes</h3>
          <p className="text-sm text-gray-500 mt-1">
            Try a different search term.
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {filteredRecipes.map((recipe) => (
            <RecipeCard
              key={recipe.id}
              recipe={recipe}
              onEdit={handleEditClick}
              onDelete={handleDeleteClick}
            />
          ))}
        </div>
      )}

      {/* Create/Edit Modal */}
      {showForm &&
        renderModal(
          editingRecipe ? 'Edit Recipe' : 'Add Recipe',
          <RecipeForm
            recipe={editingRecipe}
            onSubmit={handleFormSubmit}
            onCancel={handleFormCancel}
            loading={createLoading || updateLoading}
          />,
          handleFormCancel
        )}

      {/* Delete Confirmation */}
      <ConfirmDialog
        show={!!deletingRecipe}
        title="Delete Recipe"
        message={`Are you sure you want to delete "${deletingRecipe?.name}"? This action cannot be undone.`}
        confirmText="Delete"
        cancelText="Cancel"
        confirmVariant="danger"
        onConfirm={handleDeleteConfirm}
        onCancel={handleDeleteCancel}
        loading={deleteLoading}
      />
    </div>
  );
}
