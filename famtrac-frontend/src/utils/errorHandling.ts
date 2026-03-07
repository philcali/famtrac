export interface ErrorInfo {
  message: string;
  type: 'network' | 'http' | 'validation' | 'application';
  statusCode?: number;
  details?: string[];
}

export interface HttpError {
  response: {
    status: number;
    data: {
      error?: string;
      details?: string[];
    };
  };
}

function isHttpError(error: unknown): error is HttpError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'response' in error &&
    typeof (error as HttpError).response === 'object' &&
    'status' in (error as HttpError).response
  );
}

export function parseHttpError(error: HttpError): ErrorInfo {
  const statusCode = error.response.status;

  switch (statusCode) {
    case 400:
      return {
        message: error.response.data.error || 'Invalid request',
        type: 'validation',
        statusCode,
        details: error.response.data.details,
      };
    case 401:
      return {
        message: 'Authentication required',
        type: 'http',
        statusCode,
      };
    case 403:
      return {
        message: "Access denied. You don't have permission to perform this action.",
        type: 'http',
        statusCode,
      };
    case 404:
      return {
        message: 'Resource not found',
        type: 'http',
        statusCode,
      };
    case 500:
      return {
        message: 'Server error. Please try again later.',
        type: 'http',
        statusCode,
      };
    default:
      return {
        message: error.response.data.error || 'An error occurred',
        type: 'http',
        statusCode,
      };
  }
}

export function parseApiError(error: unknown): ErrorInfo {
  // Network error
  if (error instanceof TypeError && error.message.includes('fetch')) {
    return {
      message: 'Connection failed. Please check your network and try again.',
      type: 'network',
    };
  }

  // Timeout error
  if (error instanceof Error && error.name === 'AbortError') {
    return {
      message: 'Request timed out. Please try again.',
      type: 'network',
    };
  }

  // HTTP error with response
  if (isHttpError(error)) {
    return parseHttpError(error);
  }

  // Unknown error
  return {
    message: 'An unexpected error occurred. Please try again.',
    type: 'application',
  };
}
