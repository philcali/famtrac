import { useState, useCallback } from 'react';
import { Container, Row, Col, Modal } from 'react-bootstrap';
import { useNavigate } from 'react-router-dom';
import { FamilyList } from '../components/families/FamilyList';
import { FamilyForm } from '../components/families/FamilyForm';
import { ConfirmDialog } from '../components/common/ConfirmDialog';
import { SuccessMessage } from '../components/common/SuccessMessage';
import { Button } from '../components/common/Button';
import { useAuth } from '../auth/useAuth';
import { useApi, useApiMutation } from '../hooks/useApi';
import { createApiClient } from '../api/client';
import { getFamilies, createFamily, updateFamily, deleteFamily } from '../api/families';
import type { Family } from '../types/domain';

/**
 * FamiliesPage - Main page for managing families
 * - Lists all families (Requirement 3.1)
 * - Handles create, update, delete operations (Requirements 2.1, 4.1, 5.1)
 * - Displays user feedback for operations (Requirements 2.4, 2.5, 4.4, 5.4)
 * - Provides navigation to family details (Requirement 14.1)
 */
export function FamiliesPage() {
  const navigate = useNavigate();
  const { getToken } = useAuth();
  const [showForm, setShowForm] = useState(false);
  const [editingFamily, setEditingFamily] = useState<Family | undefined>();
  const [deletingFamily, setDeletingFamily] = useState<Family | undefined>();
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Fetch families
  const apiClient = createApiClient(getToken);
  const { data: familiesData, loading, error, refetch } = useApi(() => getFamilies(apiClient), []);

  const families = familiesData?.families || [];

  // Create family mutation
  const { mutate: createFamilyMutation, loading: createLoading } = useApiMutation((name: string) =>
    createFamily(apiClient, { name })
  );

  // Update family mutation
  const { mutate: updateFamilyMutation, loading: updateLoading } = useApiMutation(
    ({ id, name }: { id: string; name: string }) => updateFamily(apiClient, id, { name })
  );

  // Delete family mutation
  const { mutate: deleteFamilyMutation, loading: deleteLoading } = useApiMutation((id: string) =>
    deleteFamily(apiClient, id)
  );

  // Handlers
  const handleCreateClick = () => {
    setEditingFamily(undefined);
    setShowForm(true);
  };

  const handleEditClick = (family: Family) => {
    setEditingFamily(family);
    setShowForm(true);
  };

  const handleDeleteClick = (family: Family) => {
    setDeletingFamily(family);
  };

  const handleViewClick = (family: Family) => {
    navigate(`/families/${family.id}`);
  };

  const handleFormSubmit = async (name: string) => {
    if (editingFamily) {
      // Update existing family
      const response = await updateFamilyMutation({ id: editingFamily.id, name });
      if (!response.error) {
        setSuccessMessage('Family updated successfully');
        setShowForm(false);
        setEditingFamily(undefined);
        refetch();
      }
    } else {
      // Create new family
      const response = await createFamilyMutation(name);
      if (!response.error) {
        setSuccessMessage('Family created successfully');
        setShowForm(false);
        refetch();
      }
    }
  };

  const handleFormCancel = () => {
    setShowForm(false);
    setEditingFamily(undefined);
  };

  const handleDeleteConfirm = async () => {
    if (deletingFamily) {
      const response = await deleteFamilyMutation(deletingFamily.id);
      if (!response.error) {
        setSuccessMessage('Family deleted successfully');
        setDeletingFamily(undefined);
        refetch();
      }
    }
  };

  const handleDeleteCancel = () => {
    setDeletingFamily(undefined);
  };

  const handleSuccessClose = useCallback(() => {
    setSuccessMessage(null);
  }, []);

  return (
    <>
      <Container className="py-4">
        <Row className="mb-4">
          <Col>
            <h2 className="heading">
              Families
              <Button className="heading-right" icon="plus" onClick={handleCreateClick}>
                Create Family
              </Button>
            </h2>
          </Col>
        </Row>

        {successMessage && <SuccessMessage message={successMessage} onClose={handleSuccessClose} />}

        <FamilyList
          families={families}
          loading={loading}
          error={error || undefined}
          onEdit={handleEditClick}
          onDelete={handleDeleteClick}
          onView={handleViewClick}
        />

        {/* Create/Edit Modal */}
        <Modal show={showForm} onHide={handleFormCancel} centered>
          <Modal.Header closeButton>
            <Modal.Title>{editingFamily ? 'Edit Family' : 'Create Family'}</Modal.Title>
          </Modal.Header>
          <Modal.Body>
            <FamilyForm
              family={editingFamily}
              onSubmit={handleFormSubmit}
              onCancel={handleFormCancel}
              loading={createLoading || updateLoading}
            />
          </Modal.Body>
        </Modal>

        {/* Delete Confirmation Dialog */}
        <ConfirmDialog
          show={!!deletingFamily}
          title="Delete Family"
          message={`Are you sure you want to delete "${deletingFamily?.name}"? This action cannot be undone.`}
          confirmText="Delete"
          cancelText="Cancel"
          confirmVariant="danger"
          onConfirm={handleDeleteConfirm}
          onCancel={handleDeleteCancel}
          loading={deleteLoading}
        />
      </Container>
    </>
  );
}
