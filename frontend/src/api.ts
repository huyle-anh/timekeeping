export function getApiBase(): string {
  if (import.meta.env.VITE_API_BASE) {
    return import.meta.env.VITE_API_BASE
  }

  if (typeof window !== 'undefined') {
    const { protocol, port, origin } = window.location

    // When the app is served by the Rust backend (for example on :3001),
    // use the current origin so requests stay same-origin.
    if ((protocol === 'http:' || protocol === 'https:') && port !== '5173') {
      return origin
    }
  }

  // Vite dev server runs on :5173 while the backend API stays on :3001.
  return 'http://localhost:3001'
}

export async function parseJsonResponse<T>(response: Response): Promise<T> {
  const contentType = response.headers.get('content-type') ?? ''

  if (!contentType.toLowerCase().includes('application/json')) {
    const text = await response.text()
    if (text.trim().startsWith('<!DOCTYPE') || text.trim().startsWith('<html')) {
      throw new Error(
        'API returned HTML instead of JSON. Check backend/API_BASE (expected :3001 API).',
      )
    }
    throw new Error('API returned non-JSON response.')
  }

  return response.json() as Promise<T>
}

export async function parseApiError(response: Response): Promise<string> {
  try {
    const data = await parseJsonResponse<{ error?: string }>(response)
    return data.error || `HTTP ${response.status}`
  } catch {
    return `HTTP ${response.status}: Backend API is unreachable or returned invalid response.`
  }
}