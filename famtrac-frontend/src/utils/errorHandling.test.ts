import { describe, test, expect } from 'vitest';
import { parseApiError, parseHttpError, type HttpError } from './errorHandling';

/**
 * Tests for HTTP status-specific error handling
 * Validates: Requirements 15.1, 15.2, 15.3, 15.4, 15.5, 15.6
 */
describe('parseHttpError', () => {
  test('400 returns validation error message (Requirement 15.1)', () => {
    const error: HttpError = {
      response: { status: 400, data: { error: 'Name is required' } },
    };
    const result = parseHttpError(error);
    expect(result.message).toBe('Name is required');
    expect(result.type).toBe('validation');
    expect(result.statusCode).toBe(400);
  });

  test('400 with details includes them (Requirement 15.1)', () => {
    const error: HttpError = {
      response: { status: 400, data: { error: 'Validation failed', details: ['Name too short'] } },
    };
    const result = parseHttpError(error);
    expect(result.details).toEqual(['Name too short']);
  });

  test('400 without error field falls back to default (Requirement 15.1)', () => {
    const error: HttpError = {
      response: { status: 400, data: {} },
    };
    const result = parseHttpError(error);
    expect(result.message).toBe('Invalid request');
  });

  test('401 returns authentication required (Requirement 15.2)', () => {
    const error: HttpError = {
      response: { status: 401, data: {} },
    };
    const result = parseHttpError(error);
    expect(result.message).toBe('Authentication required');
    expect(result.type).toBe('http');
    expect(result.statusCode).toBe(401);
  });

  test('403 returns access denied message (Requirement 15.3)', () => {
    const error: HttpError = {
      response: { status: 403, data: {} },
    };
    const result = parseHttpError(error);
    expect(result.message).toBe("Access denied. You don't have permission to perform this action.");
    expect(result.type).toBe('http');
    expect(result.statusCode).toBe(403);
  });

  test('404 returns resource not found (Requirement 15.4)', () => {
    const error: HttpError = {
      response: { status: 404, data: {} },
    };
    const result = parseHttpError(error);
    expect(result.message).toBe('Resource not found');
    expect(result.type).toBe('http');
    expect(result.statusCode).toBe(404);
  });

  test('500 returns server error message (Requirement 15.5)', () => {
    const error: HttpError = {
      response: { status: 500, data: {} },
    };
    const result = parseHttpError(error);
    expect(result.message).toBe('Server error. Please try again later.');
    expect(result.type).toBe('http');
    expect(result.statusCode).toBe(500);
  });

  test('unknown status code uses error field from response', () => {
    const error: HttpError = {
      response: { status: 502, data: { error: 'Bad gateway' } },
    };
    const result = parseHttpError(error);
    expect(result.message).toBe('Bad gateway');
    expect(result.statusCode).toBe(502);
  });

  test('unknown status code without error field falls back', () => {
    const error: HttpError = {
      response: { status: 503, data: {} },
    };
    const result = parseHttpError(error);
    expect(result.message).toBe('An error occurred');
  });
});

describe('parseApiError', () => {
  test('network error returns connection failed (Requirement 15.6)', () => {
    const error = new TypeError('Failed to fetch');
    const result = parseApiError(error);
    expect(result.message).toBe('Connection failed. Please check your network and try again.');
    expect(result.type).toBe('network');
  });

  test('timeout error returns timeout message (Requirement 16.6)', () => {
    const error = new DOMException('The operation was aborted', 'AbortError');
    const result = parseApiError(error);
    expect(result.message).toBe('Request timed out. Please try again.');
    expect(result.type).toBe('network');
  });

  test('HTTP error delegates to parseHttpError', () => {
    const error = {
      response: { status: 403, data: {} },
    };
    const result = parseApiError(error);
    expect(result.message).toBe("Access denied. You don't have permission to perform this action.");
    expect(result.statusCode).toBe(403);
  });

  test('unknown error returns generic message', () => {
    const error = new Error('Something weird happened');
    const result = parseApiError(error);
    expect(result.message).toBe('An unexpected error occurred. Please try again.');
    expect(result.type).toBe('application');
  });
});
