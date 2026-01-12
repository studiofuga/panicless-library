// Load runtime configuration from config.json
let config = null

export async function loadConfig() {
  if (config) return config

  try {
    const response = await fetch('/config.json')
    if (!response.ok) {
      throw new Error(`Failed to load config: ${response.statusText}`)
    }
    config = await response.json()
  } catch (error) {
    console.warn('Failed to load runtime config, using defaults:', error)
    config = {
      apiBaseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080',
    }
  }

  return config
}

export function getApiBaseURL() {
  if (!config) {
    throw new Error('Config not loaded. Call loadConfig() first.')
  }
  return config.apiBaseURL
}
