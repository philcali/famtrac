import { useState, useEffect } from 'react';
import { useAuth } from '../auth/useAuth';
import { createApiClient } from '../api/client';
import { getActivities } from '../api/activities';
import type { ActivityResponse } from '../api/types';

export interface UseReportDataResult {
  activities: ActivityResponse[];
  loading: boolean;
  error: string | null;
}

/**
 * Hook that fetches all paginated activities for a dependent within a date range.
 * Unlike the "Load More" pattern used elsewhere, this hook accumulates all pages
 * upfront since reporting needs the complete dataset for summaries and charts.
 *
 * Follows the same useCallback + useEffect pattern as useApi.
 */
export function useReportData(
  familyId: string,
  dependentId: string,
  startDate: string,
  endDate: string
): UseReportDataResult {
  const { getToken } = useAuth();
  const apiClient = createApiClient(getToken);

  const [activities, setActivities] = useState<ActivityResponse[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      setLoading(true);
      setError(null);
      setActivities([]);

      let allActivities: ActivityResponse[] = [];
      let nextToken: string | undefined = undefined;

      do {
        const response = await getActivities(apiClient, familyId, dependentId, {
          startDate,
          endDate,
          next_token: nextToken,
        });

        if (cancelled) return;

        if (response.error) {
          setError(response.error);
          setLoading(false);
          return;
        }

        if (response.data) {
          allActivities = [...allActivities, ...response.data.activities];
          nextToken = response.data.next_token ?? undefined;
        }
      } while (nextToken);

      if (!cancelled) {
        setActivities(allActivities);
        setLoading(false);
      }
    };

    load();

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [startDate, endDate]);

  return { activities, loading, error };
}
