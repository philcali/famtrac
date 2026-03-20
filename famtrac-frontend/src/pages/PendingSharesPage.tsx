import { useState, useCallback, useMemo } from 'react';
import { Container, Row, Col } from 'react-bootstrap';
import { ShareCard } from '../components/shares/ShareCard';
import { SuccessMessage } from '../components/common/SuccessMessage';
import { ErrorMessage } from '../components/common/ErrorMessage';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { Button } from '../components/common/Button';
import { useAuth } from '../auth/useAuth';
import { useApi, useApiMutation } from '../hooks/useApi';
import { createApiClient } from '../api/client';
import { getSharesForAccepter, acceptShare } from '../api/shares';
import type { Share, PermissionAction } from '../types/domain';

function mapShares(raw: { permission_scope: { actions: string[] }; status: string }[]): Share[] {
  return raw.map((s) => ({
    ...s,
    permission_scope: { actions: s.permission_scope.actions as PermissionAction[] },
    status: s.status as Share['status'],
  })) as Share[];
}

/**
 * PendingSharesPage - View and accept pending share invitations
 * - Fetches shares for the authenticated user (Requirement 8.1)
 * - Displays each share with ShareCard (Requirement 8.2)
 * - Accept button calls acceptShare (Requirements 8.3, 8.4)
 * - Shows error messages from API (Requirement 8.5)
 * - Shows empty state when no shares (Requirement 8.6)
 * - Load More pagination (Requirements 8.7, 8.8)
 */
export function PendingSharesPage() {
  const { getToken } = useAuth();
  const apiClient = createApiClient(getToken);

  const [extraShares, setExtraShares] = useState<Share[]>([]);
  const [lastNextToken, setLastNextToken] = useState<string | null>(null);
  const [loadingMoreShares, setLoadingMoreShares] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [acceptError, setAcceptError] = useState<string | null>(null);

  // Fetch shares for the authenticated user
  const {
    data: sharesData,
    loading,
    error,
    refetch,
  } = useApi(() => getSharesForAccepter(apiClient), []);

  // Derive shares from fetched data + any extra pages loaded via "Load More"
  const initialShares = useMemo(
    () => (sharesData ? mapShares(sharesData.shares) : []),
    [sharesData]
  );
  const shares = useMemo(() => [...initialShares, ...extraShares], [initialShares, extraShares]);
  const nextToken = lastNextToken ?? sharesData?.next_token ?? null;

  // Accept share mutation
  const { mutate: acceptShareMutation } = useApiMutation((shareId: string) =>
    acceptShare(apiClient, shareId)
  );

  const handleAccept = async (share: Share) => {
    setAcceptError(null);
    const response = await acceptShareMutation(share.id);
    if (response.error) {
      setAcceptError(response.error);
    } else {
      setSuccessMessage('Share accepted successfully');
      setExtraShares([]);
      setLastNextToken(null);
      refetch();
    }
  };

  const handleLoadMore = async () => {
    if (!nextToken) return;
    setLoadingMoreShares(true);
    const response = await getSharesForAccepter(apiClient, { next_token: nextToken });
    if (response.data) {
      setExtraShares((prev) => [...prev, ...mapShares(response.data!.shares)]);
      setLastNextToken(response.data.next_token ?? null);
    }
    setLoadingMoreShares(false);
  };

  const handleSuccessClose = useCallback(() => {
    setSuccessMessage(null);
  }, []);

  if (loading) {
    return (
      <Container className="py-4">
        <LoadingSpinner />
      </Container>
    );
  }

  if (error) {
    return (
      <Container className="py-4">
        <h1>Shared With Me</h1>
        <ErrorMessage message={error} />
      </Container>
    );
  }

  return (
    <Container className="py-4">
      <Row className="mb-4">
        <Col>
          <h1>Shared With Me</h1>
        </Col>
      </Row>

      {successMessage && <SuccessMessage message={successMessage} onClose={handleSuccessClose} />}
      {acceptError && (
        <ErrorMessage message={acceptError} dismissible onClose={() => setAcceptError(null)} />
      )}

      {shares.length === 0 ? (
        <p>No shared families found.</p>
      ) : (
        <>
          <Row>
            {shares.map((share) => (
              <Col key={share.id} md={6} lg={4} className="mb-3">
                <ShareCard share={share} onAccept={handleAccept} />
              </Col>
            ))}
          </Row>

          {nextToken && shares.length > 0 && (
            <div className="text-center mt-3">
              <Button
                onClick={handleLoadMore}
                loading={loadingMoreShares}
                disabled={loadingMoreShares}
                variant="secondary"
              >
                Load More
              </Button>
            </div>
          )}
        </>
      )}
    </Container>
  );
}
