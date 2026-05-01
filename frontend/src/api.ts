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