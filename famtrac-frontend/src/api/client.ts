import { config } from '../config/environment';
import { parseApiError, parseHttpError } from '../utils/errorHandling';

export interface ApiClientConfig {
  baseURL: string;
  timeout: number;
}

export interface ApiResponse<T> {
  data?: T;
  error?: string;
}

export class ApiClient {
  private config: ApiClientConfig;
  private getAuthToken: () => Promise<string | null>;

  constructor(clientConfig: ApiClientConfig, getAuthToken: () => Promise<string | null>) {
    this.config = clientConfig;
    this.getAuthToken = getAuthToken;
  }

  async get<T>(path: string): Promise<ApiResponse<T>> {
    return this.request<T>('GET', path);
  }

  async post<T>(path: string, body: unknown): Promise<ApiResponse<T>> {
    return this.request<T>('POST', path, body);
  }

  async put<T>(path: string, body: unknown): Promise<ApiResponse<T>> {
    return this.request<T>('PUT', path, body);
  }

  async delete<T>(path: string): Promise<ApiResponse<T>> {
    return this.request<T>('DELETE', path);
  }

  private async request<T>(method: string, path: string, body?: unknown): Promise<ApiResponse<T>> {
    try {
      // Set up timeout handling with AbortController
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);

      // Build headers
      const headers = await this.buildHeaders(body !== undefined);

      // Construct full URL
      const url = `${this.config.baseURL}${path}`;

      // Make the request
      const response = await fetch(url, {
        method,
        headers,
        body: body ? JSON.stringify(body) : undefined,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      // Handle 401 specially - trigger re-authentication (Requirement 15.2)
      if (response.status === 401) {
        window.dispatchEvent(new CustomEvent('auth:expired'));
        return { error: 'Authentication required' };
      }

      // Parse JSON response
      const data = await response.json();

      // Handle error responses using centralized error handling (Requirements 15.1, 15.3, 15.4, 15.5)
      if (!response.ok) {
        const errorInfo = parseHttpError({
          response: {
            status: response.status,
            data: data,
          },
        });
        console.error(`API error ${response.status}:`, data);
        return { error: errorInfo.message };
      }

      return { data };
    } catch (error) {
      console.error('Request failed:', error);

      // Use centralized error parsing for network/timeout errors (Requirements 15.6, 16.6)
      const errorInfo = parseApiError(error);
      return { error: errorInfo.message };
    }
  }

  private async buildHeaders(hasBody: boolean): Promise<Record<string, string>> {
    const headers: Record<string, string> = {};

    // Add Authorization header with token
    const token = await this.getAuthToken();
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    // Add Content-Type header for requests with body
    if (hasBody) {
      headers['Content-Type'] = 'application/json';
    }

    return headers;
  }
}

// Default timeout: 30 seconds
const DEFAULT_TIMEOUT = 30000;

// Factory function to create API client with default configuration
export function createApiClient(getAuthToken: () => Promise<string | null>): ApiClient {
  return new ApiClient(
    {
      baseURL: config.apiBaseUrl,
      timeout: DEFAULT_TIMEOUT,
    },
    getAuthToken
  );
}
