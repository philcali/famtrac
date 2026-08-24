import { useState, useEffect, useCallback } from 'react';
import { type ApiResponse } from '../api/client';

export interface ApiState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
}

/**
 * Hook for fetching data from the API with loading and error states.
 * Automatically executes the API call on mount and when dependencies change.
 *
 * @param apiCall - Function that returns a Promise with ApiResponse
 * @param dependencies - Optional array of dependencies to trigger refetch
 * @returns Object with data, loading, error states and refetch function
 */
export function useApi<T>(
  apiCall: () => Promise<ApiResponse<T>>,
  dependencies: unknown[] = []
): ApiState<T> & { refetch: () => Promise<void> } {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);

    const response = await apiCall();

    if (response.error) {
      setError(response.error);
      setData(null);
    } else if (response.data) {
      setData(response.data);
      setError(null);
    }

    setLoading(false);
  }, [apiCall]);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      setLoading(true);
      setError(null);

      const response = await apiCall();

      if (cancelled) return;

      if (response.error) {
        setError(response.error);
        setData(null);
      } else if (response.data) {
        setData(response.data);
        setError(null);
      }

      setLoading(false);
    };

    load();

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, dependencies);

  return {
    data,
    loading,
    error,
    refetch: fetchData,
  };
}

/**
 * Hook for API mutations (create, update, delete operations).
 * Does not execute automatically - call mutate() to trigger the operation.
 *
 * @param apiCall - Function that takes parameters and returns a Promise with ApiResponse
 * @returns Object with mutate function, loading and error states
 */
export function useApiMutation<T, P>(
  apiCall: (params: P) => Promise<ApiResponse<T>>
): {
  mutate: (params: P) => Promise<ApiResponse<T>>;
  loading: boolean;
  error: string | null;
} {
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const mutate = useCallback(
    async (params: P): Promise<ApiResponse<T>> => {
      setLoading(true);
      setError(null);

      const response = await apiCall(params);

      if (response.error) {
        setError(response.error);
      } else {
        setError(null);
      }

      setLoading(false);
      return response;
    },
    [apiCall]
  );

  return {
    mutate,
    loading,
    error,
  };
}
