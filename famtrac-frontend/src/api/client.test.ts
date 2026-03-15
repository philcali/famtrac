import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';
import { ApiClient } from './client';

const TEST_CONFIG = { baseURL: 'http://localhost:8080', timeout: 5000 };
const mockGetToken = vi.fn().mockResolvedValue('test-token');

function createClient() {
  return new ApiClient(TEST_CONFIG, mockGetToken);
}

describe('ApiClient HTTP status-specific error handling', () => {
  const originalFetch = globalThis.fetch;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
  });

  test('401 dispatches auth:expired event and returns auth error (Requirement 15.2)', async () => {
    const dispatchSpy = vi.spyOn(window, 'dispatchEvent');
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 401,
      json: async () => ({ error: 'Unauthorized' }),
    });

    const client = createClient();
    const result = await client.get('/test');

    expect(result.error).toBe('Authentication required');
    expect(dispatchSpy).toHaveBeenCalledWith(expect.objectContaining({ type: 'auth:expired' }));
    dispatchSpy.mockRestore();
  });

  test('403 returns access denied message (Requirement 15.3)', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 403,
      json: async () => ({}),
    });

    const client = createClient();
    const result = await client.get('/test');

    expect(result.error).toBe("Access denied. You don't have permission to perform this action.");
  });

  test('404 returns resource not found (Requirement 15.4)', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      json: async () => ({}),
    });

    const client = createClient();
    const result = await client.get('/families/999');

    expect(result.error).toBe('Resource not found');
  });

  test('500 returns server error message (Requirement 15.5)', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 500,
      json: async () => ({}),
    });

    const client = createClient();
    const result = await client.get('/test');

    expect(result.error).toBe('Server error. Please try again later.');
  });

  test('400 returns validation error from response body (Requirement 15.1)', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 400,
      json: async () => ({ error: 'Name is required' }),
    });

    const client = createClient();
    const result = await client.post('/families', { name: '' });

    expect(result.error).toBe('Name is required');
  });

  test('network error returns connection failed (Requirement 15.6)', async () => {
    globalThis.fetch = vi.fn().mockRejectedValue(new TypeError('Failed to fetch'));

    const client = createClient();
    const result = await client.get('/test');

    expect(result.error).toBe('Connection failed. Please check your network and try again.');
  });

  test('timeout returns timeout message (Requirement 16.6)', async () => {
    globalThis.fetch = vi
      .fn()
      .mockRejectedValue(new DOMException('The operation was aborted', 'AbortError'));

    const client = createClient();
    const result = await client.get('/test');

    expect(result.error).toBe('Request timed out. Please try again.');
  });

  test('successful response returns data', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => ({ id: '1', name: 'Test Family' }),
    });

    const client = createClient();
    const result = await client.get('/families/1');

    expect(result.data).toEqual({ id: '1', name: 'Test Family' });
    expect(result.error).toBeUndefined();
  });

  test('all errors are logged to console (Requirement 15.7)', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 500,
      json: async () => ({ error: 'Internal error' }),
    });

    const client = createClient();
    await client.get('/test');

    expect(consoleSpy).toHaveBeenCalled();
    consoleSpy.mockRestore();
  });
});
