import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Container, Row, Col, Card, Modal } from 'react-bootstrap';
import { ErrorMessage } from '../components/common/ErrorMessage';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { SuccessMessage } from '../components/common/SuccessMessage';
import { ConfirmDialog } from '../components/common/ConfirmDialog';
import { Button } from '../components/common/Button';
import { ActivityList } from '../components/activities/ActivityList';
import { ActivityForm } from '../components/activities/ActivityForm';
import { ActivityFilters } from '../components/activities/ActivityFilters';
import { formatAge } from '../utils/dateUtils';
import { useAuth } from '../auth/useAuth';
import { useApi, useApiMutation } from '../hooks/useApi';
import { createApiClient } from '../api/client';
import { getDependent } from '../api/dependents';
import { getActivities, createActivity, updateActivity, deleteActivity } from '../api/activities';
import type { ActivityType } from '../types/domain';
import type { CreateActivityRequest, UpdateActivityRequest, ActivityResponse } from '../api/types';

/**
 * DependentDetailPage - View single dependent with activities
 * - Displays dependent information with age (Requirements 7.2, 7.3, 7.4)
 * - Displays activity list with filtering (Requirements 11.1, 11.2)
 * - Supports activity creation, editing, and deletion (Requirements 10.1, 10.10, 12.1, 12.4, 13.1, 13.4)
 * - Implements filtering controls (Requirements 11.5, 11.6)
 * - Provides navigation back to family (Requirement 14.3)
 */
export function DependentDetailPage() {
  const { dependentId } = useParams<{ dependentId: string }>();
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

  // Fetch dependent details
  const {
    data: dependent,
    loading: dependentLoading,
    error: dependentError,
  } = useApi(() => getDependent(apiClient, dependentId ?? 'NA'), [dependentId]);

  // Fetch activities with filters
  const {
    data: activitiesData,
    loading: activitiesLoading,
    error: activitiesError,
    refetch: refetchActivities,
  } = useApi(
    () =>
      getActivities(apiClient, dependentId ?? 'NA', {
        startDate: startDate || undefined,
        endDate: endDate || undefined,
        activityType: activityTypeFilter || undefined,
      }),
    [dependentId, startDate, endDate, activityTypeFilter]
  );

  // Mutations
  const { mutate: createActivityMutation, loading: createLoading } = useApiMutation<
    ActivityResponse,
    CreateActivityRequest
  >((data) => createActivity(apiClient, data));

  const { mutate: updateActivityMutation, loading: updateLoading } = useApiMutation<
    ActivityResponse,
    { id: string; data: UpdateActivityRequest }
  >(({ id, data }) => updateActivity(apiClient, id, data));

  const { mutate: deleteActivityMutation, loading: deleteLoading } = useApiMutation<void, string>(
    (id) => deleteActivity(apiClient, id)
  );

  // Handlers
  const handleBackClick = () => {
    if (dependent) {
      navigate(`/families/${dependent.family_id}`);
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

  const handleActivityFormSubmit = async (data: CreateActivityRequest) => {
    if (editingActivity) {
      // Update existing activity
      const updateData: UpdateActivityRequest = {
        timestamp: data.timestamp,
        feeding_type: data.feeding_type,
        contents: data.contents,
        start_time: data.start_time,
        end_time: data.end_time,
        volume_ml: data.volume_ml,
      };

      const response = await updateActivityMutation({
        id: editingActivity.id,
        data: updateData,
      });

      if (!response.error) {
        setSuccessMessage('Activity updated successfully');
        setShowActivityForm(false);
        setEditingActivity(undefined);
        await refetchActivities();
      }
    } else {
      // Create new activity
      const response = await createActivityMutation(data);

      if (!response.error) {
        setSuccessMessage('Activity created successfully');
        setShowActivityForm(false);
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
  };

  if (dependentLoading) {
    return (
      <Container className="py-4">
        <LoadingSpinner />
      </Container>
    );
  }

  if (dependentError) {
    return (
      <Container className="py-4">
        <ErrorMessage message={dependentError} />
        <Button onClick={handleBackClick} className="mt-3">
          Back
        </Button>
      </Container>
    );
  }

  if (!dependent) {
    return (
      <Container className="py-4">
        <ErrorMessage message="Dependent not found" />
        <Button onClick={handleBackClick} className="mt-3">
          Back
        </Button>
      </Container>
    );
  }

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  };

  const activities = activitiesData?.activities || [];

  return (
    <Container className="py-4">
      {successMessage && (
        <SuccessMessage message={successMessage} onClose={() => setSuccessMessage(null)} />
      )}

      <Row className="mb-4">
        <Col>
          <Button variant="secondary" onClick={handleBackClick} className="mb-3">
            ← Back to Family
          </Button>
          <h1>{dependent.name}</h1>
        </Col>
      </Row>

      {/* Dependent Information Card */}
      <Card className="mb-4">
        <Card.Body>
          <Card.Title>Dependent Information</Card.Title>
          <Card.Text>
            <strong>Age:</strong> {formatAge(dependent.date_of_birth)}
            <br />
            <strong>Date of Birth:</strong> {formatDate(dependent.date_of_birth)}
            <br />
            <br />
            <strong>Created:</strong> {formatDate(dependent.created_at)}
            <br />
            <strong>Updated:</strong> {formatDate(dependent.updated_at)}
          </Card.Text>
        </Card.Body>
      </Card>

      {/* Activities Section */}
      <Row className="mb-3">
        <Col>
          <h2>Activities</h2>
        </Col>
        <Col className="text-end">
          <Button variant="primary" onClick={handleAddActivity}>
            Add Activity
          </Button>
        </Col>
      </Row>

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
        onEdit={handleEditActivity}
        onDelete={handleDeleteActivity}
      />

      {/* Activity Form Modal */}
      <Modal show={showActivityForm} onHide={handleActivityFormCancel} size="lg">
        <Modal.Header closeButton>
          <Modal.Title>{editingActivity ? 'Edit Activity' : 'Add Activity'}</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          <ActivityForm
            activity={editingActivity}
            dependentId={dependentId!}
            onSubmit={handleActivityFormSubmit}
            onCancel={handleActivityFormCancel}
            loading={createLoading || updateLoading}
          />
        </Modal.Body>
      </Modal>

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
    </Container>
  );
}
