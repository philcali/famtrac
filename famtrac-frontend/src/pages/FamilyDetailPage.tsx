import { useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Container, Row, Col, Modal, Card } from 'react-bootstrap';
import { DependentList } from '../components/dependents/DependentList';
import { DependentForm } from '../components/dependents/DependentForm';
import { ConfirmDialog } from '../components/common/ConfirmDialog';
import { SuccessMessage } from '../components/common/SuccessMessage';
import { ErrorMessage } from '../components/common/ErrorMessage';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { Button } from '../components/common/Button';
import { useAuth } from '../auth/useAuth';
import { useApi, useApiMutation } from '../hooks/useApi';
import { createApiClient } from '../api/client';
import { getFamily } from '../api/families';
import {
  getDependents,
  createDependent,
  updateDependent,
  deleteDependent,
} from '../api/dependents';
import type { Dependent } from '../types/domain';

/**
 * FamilyDetailPage - View single family with dependents
 * - Displays family information (Requirements 3.2, 3.3)
 * - Lists associated dependents (Requirement 7.1)
 * - Supports adding, editing, and deleting dependents (Requirements 6.1, 8.1, 9.1)
 * - Provides navigation to dependent details (Requirement 14.2)
 */
export function FamilyDetailPage() {
  const { familyId } = useParams<{ familyId: string }>();
  const navigate = useNavigate();
  const { getToken } = useAuth();
  const [showForm, setShowForm] = useState(false);
  const [editingDependent, setEditingDependent] = useState<Dependent | undefined>();
  const [deletingDependent, setDeletingDependent] = useState<Dependent | undefined>();
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const apiClient = createApiClient(getToken);

  // Fetch family details
  const {
    data: family,
    loading: familyLoading,
    error: familyError,
  } = useApi(() => getFamily(apiClient, familyId ?? 'NA'), [familyId]);

  // Fetch dependents for this family
  const {
    data: dependentsData,
    loading: dependentsLoading,
    error: dependentsError,
    refetch: refetchDependents,
  } = useApi(() => getDependents(apiClient, familyId ?? 'NA'), [familyId]);

  const dependents = dependentsData?.dependents || [];

  // Create dependent mutation
  const { mutate: createDependentMutation, loading: createLoading } = useApiMutation(
    (data: { name: string; date_of_birth: string; family_id: string }) =>
      createDependent(apiClient, familyId ?? 'NA', data)
  );

  // Update dependent mutation
  const { mutate: updateDependentMutation, loading: updateLoading } = useApiMutation(
    ({ id, data }: { id: string; data: { name: string; date_of_birth: string } }) =>
      updateDependent(apiClient, familyId ?? 'NA', id, data)
  );

  // Delete dependent mutation
  const { mutate: deleteDependentMutation, loading: deleteLoading } = useApiMutation((id: string) =>
    deleteDependent(apiClient, familyId ?? 'NA', id)
  );

  // Handlers
  const handleCreateClick = () => {
    setEditingDependent(undefined);
    setShowForm(true);
  };

  const handleEditClick = (dependent: Dependent) => {
    setEditingDependent(dependent);
    setShowForm(true);
  };

  const handleDeleteClick = (dependent: Dependent) => {
    setDeletingDependent(dependent);
  };

  const handleViewClick = (dependent: Dependent) => {
    navigate(`/families/${familyId}/dependents/${dependent.id}`);
  };

  const handleFormSubmit = async (data: {
    name: string;
    date_of_birth: string;
    family_id: string;
  }) => {
    if (editingDependent) {
      // Update existing dependent
      const response = await updateDependentMutation({
        id: editingDependent.id,
        data: { name: data.name, date_of_birth: data.date_of_birth },
      });
      if (!response.error) {
        setSuccessMessage('Dependent updated successfully');
        setShowForm(false);
        setEditingDependent(undefined);
        refetchDependents();
      }
    } else {
      // Create new dependent
      const response = await createDependentMutation(data);
      if (!response.error) {
        setSuccessMessage('Dependent created successfully');
        setShowForm(false);
        refetchDependents();
      }
    }
  };

  const handleFormCancel = () => {
    setShowForm(false);
    setEditingDependent(undefined);
  };

  const handleDeleteConfirm = async () => {
    if (deletingDependent) {
      const response = await deleteDependentMutation(deletingDependent.id);
      if (!response.error) {
        setSuccessMessage('Dependent deleted successfully');
        setDeletingDependent(undefined);
        refetchDependents();
      }
    }
  };

  const handleDeleteCancel = () => {
    setDeletingDependent(undefined);
  };

  const handleSuccessClose = useCallback(() => {
    setSuccessMessage(null);
  }, []);

  const handleBackClick = () => {
    navigate('/');
  };

  if (familyLoading) {
    return (
      <Container className="py-4">
        <LoadingSpinner />
      </Container>
    );
  }

  if (familyError) {
    return (
      <Container className="py-4">
        <ErrorMessage message={familyError} />
        <Button onClick={handleBackClick} className="mt-3">
          Back to Families
        </Button>
      </Container>
    );
  }

  if (!family) {
    return (
      <Container className="py-4">
        <ErrorMessage message="Family not found" />
        <Button onClick={handleBackClick} className="mt-3">
          Back to Families
        </Button>
      </Container>
    );
  }

  return (
    <Container className="py-4">
      <Row className="mb-4">
        <Col>
          <Button variant="secondary" onClick={handleBackClick} className="mb-3">
            ← Back to Families
          </Button>
          <h1>{family.name}</h1>
        </Col>
      </Row>

      {/* Family Information Card */}
      <Card className="mb-4">
        <Card.Body>
          <Card.Title>Family Information</Card.Title>
          <Card.Text>
            <strong>Created:</strong> {new Date(family.created_at).toLocaleDateString()}
            <br />
            <strong>Updated:</strong> {new Date(family.updated_at).toLocaleDateString()}
          </Card.Text>
        </Card.Body>
      </Card>

      {/* Dependents Section */}
      <Row className="mb-3">
        <Col>
          <h2>Dependents</h2>
        </Col>
        <Col xs="auto">
          <Button onClick={handleCreateClick}>Add Dependent</Button>
        </Col>
      </Row>

      {successMessage && <SuccessMessage message={successMessage} onClose={handleSuccessClose} />}

      <DependentList
        dependents={dependents}
        loading={dependentsLoading}
        error={dependentsError || undefined}
        onEdit={handleEditClick}
        onDelete={handleDeleteClick}
        onView={handleViewClick}
      />

      {/* Create/Edit Modal */}
      <Modal show={showForm} onHide={handleFormCancel} centered>
        <Modal.Header closeButton>
          <Modal.Title>{editingDependent ? 'Edit Dependent' : 'Add Dependent'}</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          <DependentForm
            dependent={editingDependent}
            familyId={familyId ?? 'NA'}
            onSubmit={handleFormSubmit}
            onCancel={handleFormCancel}
            loading={createLoading || updateLoading}
          />
        </Modal.Body>
      </Modal>

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        show={!!deletingDependent}
        title="Delete Dependent"
        message={`Are you sure you want to delete "${deletingDependent?.name}"? This action cannot be undone.`}
        confirmText="Delete"
        cancelText="Cancel"
        confirmVariant="danger"
        onConfirm={handleDeleteConfirm}
        onCancel={handleDeleteCancel}
        loading={deleteLoading}
      />
    </Container>
  );
}
