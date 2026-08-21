import { useState, useMemo } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { ErrorMessage } from '../components/common/ErrorMessage';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { SuccessMessage } from '../components/common/SuccessMessage';
import { ConfirmDialog } from '../components/common/ConfirmDialog';
import { Button } from '../components/common/Button';
import { ActivityList } from '../components/activities/ActivityList';
import { ActivityForm } from '../components/activities/ActivityForm';
import { ActivityFilters } from '../components/activities/ActivityFilters';
import { useAuth } from '../auth/useAuth';
import { useApi, useApiMutation } from '../hooks/useApi';
import { createApiClient } from '../api/client';
import { getDependent } from '../api/dependents';
import { getActivities, createActivity, updateActivity, deleteActivity } from '../api/activities';
import type { ActivityType } from '../types/domain';
import type { CreateActivityRequest, UpdateActivityRequest, ActivityResponse } from '../api/types';
import { DependentCard } from '../components/dependents/DependentCard';

/**
 * DependentDetailPage - View single dependent with activities
 * - Displays dependent information with age (Requirements 7.2, 7.3, 7.4)
 * - Displays activity list with filtering (Requirements 11.1, 11.2)
 * - Supports activity creation, editing, and deletion (Requirements 10.1, 10.10, 12.1, 12.4, 13.1, 13.4)
 * - Implements filtering controls (Requirements 11.5, 11.6)
 * - Provides navigation back to family (Requirement 14.3)
 */
export function DependentDetailPage() {
  const { familyId, dependentId } = useParams<{ familyId: string; dependentId: string }>();
  const navigate = useNavigate();
  const { getToken } = useAuth();

  const apiClient = createApiClient(getToken);

  // UI state
  const [showActivityForm, setShowActivityForm] = useState(false);
  const [editingActivity, setEditingActivity] = useState<ActivityResponse | undefined>(undefined);
  const [deletingActivity, setDeletingActivity] = useState<ActivityResponse | undefined>(undefined);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Filter state
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  const [activityTypeFilter, setActivityTypeFilter] = useState<ActivityType | ''>('');

  // Pagination state
  const [extraActivities, setExtraActivities] = useState<ActivityResponse[]>([]);
  const [lastActivitiesNextToken, setLastActivitiesNextToken] = useState<string | null>(null);
  const [loadingMoreActivities, setLoadingMoreActivities] = useState(false);

  // Fetch dependent details
  const {
    data: dependent,
    loading: dependentLoading,
    error: dependentError,
  } = useApi(
    () => getDependent(apiClient, familyId ?? 'NA', dependentId ?? 'NA'),
    [familyId, dependentId]
  );

  // Fetch activities with filters
  const {
    data: activitiesData,
    loading: activitiesLoading,
    error: activitiesError,
    refetch: refetchActivities,
  } = useApi(
    () =>
      getActivities(apiClient, familyId ?? 'NA', dependentId ?? 'NA', {
        startDate: startDate || undefined,
        endDate: endDate || undefined,
        activityType: activityTypeFilter || undefined,
      }),
    [familyId, dependentId, startDate, endDate, activityTypeFilter]
  );

  // Mutations
  const { mutate: createActivityMutation, loading: createLoading } = useApiMutation<
    ActivityResponse,
    CreateActivityRequest
  >((data) => createActivity(apiClient, familyId ?? 'NA', dependentId ?? 'NA', data));

  const { mutate: updateActivityMutation, loading: updateLoading } = useApiMutation<
    ActivityResponse,
    { id: string; data: UpdateActivityRequest }
  >(({ id, data }) => updateActivity(apiClient, familyId ?? 'NA', dependentId ?? 'NA', id, data));

  const { mutate: deleteActivityMutation, loading: deleteLoading } = useApiMutation<void, string>(
    (id) => deleteActivity(apiClient, familyId ?? 'NA', dependentId ?? 'NA', id)
  );

  // Handlers
  const handleBackClick = () => {
    if (familyId) {
      navigate(`/families/${familyId}`);
    } else {
      navigate('/');
    }
  };

  const handleAddActivity = () => {
    setEditingActivity(undefined);
    setShowActivityForm(true);
  };

  const handleEditActivity = (activity: ActivityResponse) => {
    setEditingActivity(activity);
    setShowActivityForm(true);
  };

  const handleDeleteActivity = (activity: ActivityResponse) => {
    setDeletingActivity(activity);
  };

  const handleUpdateActivity = async (
    data: CreateActivityRequest,
    previousActivity: ActivityResponse
  ) => {
    const updateData: UpdateActivityRequest = {
      type: data.type,
      timestamp: data.timestamp,
      feeding_type: data.feeding_type,
      contents: data.contents,
      notes: data.notes,
      description: data.description,
      end_time: data.end_time,
      volume_ml: data.volume_ml,
      start_time: data.start_time,
      medicine_added: data.medicine_added,
    };

    const response = await updateActivityMutation({
      id: previousActivity.id,
      data: updateData,
    });

    if (!response.error) {
      setSuccessMessage('Activity updated successfully');
      setShowActivityForm(false);
      setEditingActivity(undefined);
      setExtraActivities([]);
      setLastActivitiesNextToken(null);
      await refetchActivities();
    }
  };

  const handleStopTimeButton = async (previousActivity: ActivityResponse) => {
    handleUpdateActivity(
      {
        ...previousActivity,
        family_id: familyId ?? 'NA',
        end_time: new Date().toISOString(),
      },
      previousActivity
    );
  };

  const handleActivityFormSubmit = async (data: CreateActivityRequest) => {
    if (editingActivity) {
      // Update existing activity
      handleUpdateActivity(data, editingActivity);
    } else {
      // Create new activity
      const response = await createActivityMutation(data);

      if (!response.error) {
        setSuccessMessage('Activity created successfully');
        setShowActivityForm(false);
        setExtraActivities([]);
        setLastActivitiesNextToken(null);
        await refetchActivities();
      }
    }
  };

  const handleActivityFormCancel = () => {
    setShowActivityForm(false);
    setEditingActivity(undefined);
  };

  const handleConfirmDelete = async () => {
    if (deletingActivity) {
      const response = await deleteActivityMutation(deletingActivity.id);

      if (!response.error) {
        setSuccessMessage('Activity deleted successfully');
        setDeletingActivity(undefined);
        setExtraActivities([]);
        setLastActivitiesNextToken(null);
        await refetchActivities();
      }
    }
  };

  const handleCancelDelete = () => {
    setDeletingActivity(undefined);
  };

  const handleClearFilters = () => {
    setStartDate('');
    setEndDate('');
    setActivityTypeFilter('');
    setExtraActivities([]);
    setLastActivitiesNextToken(null);
  };

  const handleLoadMoreActivities = async () => {
    if (!activitiesNextToken) return;
    setLoadingMoreActivities(true);
    const response = await getActivities(apiClient, familyId ?? 'NA', dependentId ?? 'NA', {
      startDate: startDate || undefined,
      endDate: endDate || undefined,
      activityType: activityTypeFilter || undefined,
      next_token: activitiesNextToken,
    });
    if (response.data) {
      setExtraActivities((prev) => [...prev, ...response.data!.activities]);
      setLastActivitiesNextToken(response.data.next_token ?? null);
    }
    setLoadingMoreActivities(false);
  };

  const initialActivities = useMemo(() => activitiesData?.activities || [], [activitiesData]);
  const activities = useMemo(
    () => [...initialActivities, ...extraActivities],
    [initialActivities, extraActivities]
  );
  const activitiesNextToken = lastActivitiesNextToken ?? activitiesData?.next_token ?? null;

  const renderModal = (title: string, body: React.ReactNode, onClose: () => void) => (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="fixed inset-0 bg-black/30" onClick={onClose} />
      <div className="relative z-10 w-full max-w-2xl bg-white rounded-2xl shadow-xl max-h-[90vh] flex flex-col">
        <div className="flex justify-between items-center p-4 border-b border-gray-100 flex-shrink-0">
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
        <div className="p-4 overflow-y-auto">{body}</div>
      </div>
    </div>
  );

  if (dependentLoading) {
    return (
      <div className="py-4 max-w-5xl mx-auto px-4">
        <LoadingSpinner />
      </div>
    );
  }

  if (dependentError) {
    return (
      <div className="py-4 max-w-5xl mx-auto px-4">
        <ErrorMessage message={dependentError} />
        <Button onClick={handleBackClick} className="mt-3">
          ← Back to Family
        </Button>
      </div>
    );
  }

  if (!dependent) {
    return (
      <div className="py-4 max-w-5xl mx-auto px-4">
        <ErrorMessage message="Dependent not found" />
        <Button onClick={handleBackClick} className="mt-3">
          ← Back to Family
        </Button>
      </div>
    );
  }

  return (
    <div className="py-4 max-w-5xl mx-auto px-4">
      {successMessage && (
        <SuccessMessage message={successMessage} onClose={() => setSuccessMessage(null)} />
      )}

      <div className="mb-4">
        <h2 className="heading">
          {dependent.name}
          <div className="ml-auto flex items-center gap-2">
            <Button variant="secondary" onClick={handleBackClick}>
              ← Back to Family
            </Button>
            <Button
              variant="secondary"
              onClick={() => navigate(`/families/${familyId}/dependents/${dependentId}/reports`)}
            >
              Reports
            </Button>
          </div>
        </h2>
      </div>

      {/* Dependent Information Card */}
      <DependentCard dependent={dependent} overrideTitle="Dependent Information" />

      {/* Activities Section */}
      <div className="mb-3">
        <h2 className="heading">
          Activities
          <Button
            className="heading-right"
            variant="primary"
            icon="plus"
            onClick={handleAddActivity}
          >
            Add Activity
          </Button>
        </h2>
      </div>

      {/* Activity Filters */}
      <ActivityFilters
        startDate={startDate}
        endDate={endDate}
        activityType={activityTypeFilter}
        onStartDateChange={setStartDate}
        onEndDateChange={setEndDate}
        onActivityTypeChange={setActivityTypeFilter}
        onClearFilters={handleClearFilters}
      />

      {/* Activity List */}
      <ActivityList
        activities={activities}
        loading={activitiesLoading}
        error={activitiesError || undefined}
        hasMore={!!activitiesNextToken}
        loadingMore={loadingMoreActivities}
        onLoadMore={handleLoadMoreActivities}
        onEdit={handleEditActivity}
        onDelete={handleDeleteActivity}
        onStop={handleStopTimeButton}
      />

      {/* Activity Form Modal */}
      {showActivityForm &&
        renderModal(
          editingActivity ? 'Edit Activity' : 'Add Activity',
          <ActivityForm
            activity={editingActivity}
            familyId={familyId!}
            dependentId={dependentId!}
            onSubmit={handleActivityFormSubmit}
            onCancel={handleActivityFormCancel}
            loading={createLoading || updateLoading}
          />,
          handleActivityFormCancel
        )}

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        show={!!deletingActivity}
        title="Delete Activity"
        message={`Are you sure you want to delete this activity? This action cannot be undone.`}
        confirmText="Delete"
        confirmVariant="danger"
        onConfirm={handleConfirmDelete}
        onCancel={handleCancelDelete}
        loading={deleteLoading}
      />
    </div>
  );
}
