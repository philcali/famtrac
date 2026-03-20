import { useState, useCallback, useMemo } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Container, Row, Col, Modal, Card, Form } from 'react-bootstrap';
import { DependentList } from '../components/dependents/DependentList';
import { DependentForm } from '../components/dependents/DependentForm';
import { ShareList } from '../components/shares/ShareList';
import { ShareForm } from '../components/shares/ShareForm';
import { PermissionScopeSelector } from '../components/shares/PermissionScopeSelector';
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
import { getShares, createShare, updateShare, revokeShare } from '../api/shares';
import type { Dependent, Share, PermissionAction } from '../types/domain';
import type { CreateShareRequest, UpdateShareRequest } from '../api/types';

function mapShares(raw: { permission_scope: { actions: string[] }; status: string }[]): Share[] {
  return raw.map((s) => ({
    ...s,
    permission_scope: { actions: s.permission_scope.actions as PermissionAction[] },
    status: s.status as Share['status'],
  })) as Share[];
}

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

  // Share state
  const [showShareForm, setShowShareForm] = useState(false);
  const [editingShare, setEditingShare] = useState<Share | undefined>();
  const [editPermissions, setEditPermissions] = useState<PermissionAction[]>([]);
  const [revokingShare, setRevokingShare] = useState<Share | undefined>();
  const [extraShares, setExtraShares] = useState<Share[]>([]);
  const [lastSharesNextToken, setLastSharesNextToken] = useState<string | null>(null);
  const [loadingMoreShares, setLoadingMoreShares] = useState(false);

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

  // Fetch shares for this family
  const {
    data: sharesData,
    loading: sharesLoading,
    error: sharesError,
    refetch: refetchShares,
  } = useApi(() => getShares(apiClient, familyId ?? 'NA'), [familyId]);

  // Derive shares from fetched data + any extra pages loaded via "Load More"
  const initialShares = useMemo(
    () => (sharesData ? mapShares(sharesData.shares) : []),
    [sharesData]
  );
  const shares = useMemo(() => [...initialShares, ...extraShares], [initialShares, extraShares]);
  const sharesNextToken = lastSharesNextToken ?? sharesData?.next_token ?? null;

  // Share mutations
  const { mutate: createShareMutation, loading: createShareLoading } = useApiMutation(
    (data: CreateShareRequest) => createShare(apiClient, familyId ?? 'NA', data)
  );

  const { mutate: updateShareMutation, loading: updateShareLoading } = useApiMutation(
    ({ id, data }: { id: string; data: UpdateShareRequest }) => updateShare(apiClient, id, data)
  );

  const { mutate: revokeShareMutation, loading: revokeShareLoading } = useApiMutation(
    (id: string) => revokeShare(apiClient, id)
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

  // Share handlers
  const handleInviteClick = () => {
    setShowShareForm(true);
  };

  const handleShareFormSubmit = async (data: CreateShareRequest) => {
    const response = await createShareMutation(data);
    if (!response.error) {
      setSuccessMessage('Share created successfully');
      setShowShareForm(false);
      setExtraShares([]);
      setLastSharesNextToken(null);
      refetchShares();
    }
  };

  const handleShareFormCancel = () => {
    setShowShareForm(false);
  };

  const handleShareEditClick = (share: Share) => {
    setEditingShare(share);
    setEditPermissions([...share.permission_scope.actions]);
  };

  const handleShareEditSubmit = async () => {
    if (editingShare) {
      const response = await updateShareMutation({
        id: editingShare.id,
        data: { permission_scope: { actions: editPermissions } },
      });
      if (!response.error) {
        setSuccessMessage('Share updated successfully');
        setEditingShare(undefined);
        setEditPermissions([]);
        setExtraShares([]);
        setLastSharesNextToken(null);
        refetchShares();
      }
    }
  };

  const handleShareEditCancel = () => {
    setEditingShare(undefined);
    setEditPermissions([]);
  };

  const handleShareRevokeClick = (share: Share) => {
    setRevokingShare(share);
  };

  const handleShareRevokeConfirm = async () => {
    if (revokingShare) {
      const response = await revokeShareMutation(revokingShare.id);
      if (!response.error) {
        setSuccessMessage('Share revoked successfully');
        setRevokingShare(undefined);
        setExtraShares([]);
        setLastSharesNextToken(null);
        refetchShares();
      }
    }
  };

  const handleShareRevokeCancel = () => {
    setRevokingShare(undefined);
  };

  const handleLoadMoreShares = async () => {
    if (!sharesNextToken) return;
    setLoadingMoreShares(true);
    const response = await getShares(apiClient, familyId ?? 'NA', { next_token: sharesNextToken });
    if (response.data) {
      setExtraShares((prev) => [...prev, ...mapShares(response.data!.shares)]);
      setLastSharesNextToken(response.data.next_token ?? null);
    }
    setLoadingMoreShares(false);
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

      {/* Shares Section */}
      <Row className="mb-3 mt-4">
        <Col>
          <h2>Shares</h2>
        </Col>
        <Col xs="auto">
          <Button onClick={handleInviteClick}>Invite User</Button>
        </Col>
      </Row>

      <ShareList
        shares={shares}
        loading={sharesLoading}
        error={sharesError || undefined}
        hasMore={!!sharesNextToken}
        loadingMore={loadingMoreShares}
        onLoadMore={handleLoadMoreShares}
        onEdit={handleShareEditClick}
        onRevoke={handleShareRevokeClick}
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

      {/* Create Share Modal */}
      <Modal show={showShareForm} onHide={handleShareFormCancel} centered>
        <Modal.Header closeButton>
          <Modal.Title>Invite User</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          <ShareForm
            familyId={familyId ?? 'NA'}
            onSubmit={handleShareFormSubmit}
            onCancel={handleShareFormCancel}
            loading={createShareLoading}
          />
        </Modal.Body>
      </Modal>

      {/* Edit Share Permissions Modal */}
      <Modal show={!!editingShare} onHide={handleShareEditCancel} centered>
        <Modal.Header closeButton>
          <Modal.Title>Edit Permissions</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          <Form.Group className="mb-3">
            <Form.Label>Permissions</Form.Label>
            <PermissionScopeSelector value={editPermissions} onChange={setEditPermissions} />
          </Form.Group>
          <div className="d-flex gap-2">
            <Button
              variant="primary"
              onClick={handleShareEditSubmit}
              loading={updateShareLoading}
              disabled={updateShareLoading}
            >
              Save
            </Button>
            <Button
              variant="secondary"
              onClick={handleShareEditCancel}
              disabled={updateShareLoading}
            >
              Cancel
            </Button>
          </div>
        </Modal.Body>
      </Modal>

      {/* Revoke Share Confirmation Dialog */}
      <ConfirmDialog
        show={!!revokingShare}
        title="Revoke Share"
        message={`Are you sure you want to revoke the share for "${revokingShare?.accepter_email}"? This action cannot be undone.`}
        confirmText="Revoke"
        cancelText="Cancel"
        confirmVariant="danger"
        onConfirm={handleShareRevokeConfirm}
        onCancel={handleShareRevokeCancel}
        loading={revokeShareLoading}
      />
    </Container>
  );
}
