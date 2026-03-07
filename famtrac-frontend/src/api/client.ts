import { config } from '../config/environment';

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

      // Handle 401 specially - trigger re-authentication
      if (response.status === 401) {
        window.dispatchEvent(new CustomEvent('auth:expired'));
        return { error: 'Authentication required' };
      }

      // Parse JSON response
      const data = await response.json();

      // Handle error responses
      if (!response.ok) {
        console.error(`API error ${response.status}:`, data);
        return {
          error: data.error || `Request failed with status ${response.status}`,
        };
      }

      return { data };
    } catch (error) {
      console.error('Request failed:', error);

      // Handle timeout errors
      if (error instanceof Error && error.name === 'AbortError') {
        return { error: 'Request timed out. Please try again.' };
      }

      // Handle network errors
      if (error instanceof TypeError && error.message.includes('fetch')) {
        return {
          error: 'Connection failed. Please check your network and try again.',
        };
      }

      // Handle unknown errors
      return { error: 'An unexpected error occurred. Please try again.' };
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
