const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080/api/v1"
// SSO API uses the base URL without /api/v1
const SSO_API_BASE_URL = API_BASE_URL.replace("/api/v1", "")

interface ApiError {
  message: string
  status: number
}

interface TokenResponse {
  access_token: string
  token_type: string
  expires_in: number
  refresh_token?: string
}

class ApiClient {
  private baseUrl: string

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    }

    // Merge with existing headers if any
    if (options.headers) {
      const existingHeaders = options.headers as Record<string, string>
      Object.assign(headers, existingHeaders)
    }

    // Add auth token if available
    const token = this.getAuthToken()
    if (token) {
      headers["Authorization"] = `Bearer ${token}`
    }

    try {
      const response = await fetch(url, {
        ...options,
        headers,
      })

      if (!response.ok) {
        const error: ApiError = {
          message: await response.text().catch(() => "An error occurred"),
          status: response.status,
        }
        throw error
      }

      // Handle empty responses
      const contentType = response.headers.get("content-type")
      if (!contentType || !contentType.includes("application/json")) {
        return {} as T
      }

      return response.json()
    } catch (error) {
      if ((error as ApiError).status) {
        throw error
      }
      throw {
        message: "Network error. Please check your connection.",
        status: 0,
      } as ApiError
    }
  }

  private getAuthToken(): string | null {
    if (typeof window === "undefined") return null
    return localStorage.getItem("auth_token")
  }

  setAuthToken(token: string) {
    if (typeof window !== "undefined") {
      localStorage.setItem("auth_token", token)
    }
  }

  clearAuthToken() {
    if (typeof window !== "undefined") {
      localStorage.removeItem("auth_token")
    }
  }

  // Generic HTTP methods
  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: "GET" })
  }

  async post<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "POST",
      body: data ? JSON.stringify(data) : undefined,
    })
  }

  async put<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "PUT",
      body: data ? JSON.stringify(data) : undefined,
    })
  }

  async patch<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "PATCH",
      body: data ? JSON.stringify(data) : undefined,
    })
  }

  async delete<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "DELETE",
      body: data ? JSON.stringify(data) : undefined,
    })
  }

  // Upload files
  async upload<T>(endpoint: string, file: File): Promise<T> {
    const formData = new FormData()
    formData.append("file", file)

    const token = this.getAuthToken()
    const headers: Record<string, string> = {}
    if (token) {
      headers["Authorization"] = `Bearer ${token}`
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method: "POST",
      headers,
      body: formData,
    })

    if (!response.ok) {
      throw {
        message: await response.text().catch(() => "Upload failed"),
        status: response.status,
      } as ApiError
    }

    return response.json()
  }

  // SSO Methods
  async exchangeSSOCode(code: string): Promise<TokenResponse> {
    const response = await fetch(`${SSO_API_BASE_URL}/api/sso/exchange`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ code }),
    })

    if (!response.ok) {
      const error: ApiError = {
        message: await response.text().catch(() => "SSO exchange failed"),
        status: response.status,
      }
      throw error
    }

    return response.json()
  }

  async getSSOUserInfo(token: string): Promise<Record<string, unknown>> {
    const response = await fetch(`${SSO_API_BASE_URL}/api/sso/userinfo`, {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    })

    if (!response.ok) {
      const error: ApiError = {
        message: await response.text().catch(() => "Failed to get user info"),
        status: response.status,
      }
      throw error
    }

    return response.json()
  }

  clearUserData() {
    if (typeof window !== "undefined") {
      localStorage.removeItem("auth_token")
      localStorage.removeItem("auth_user")
    }
  }

  setUserData(token: string, user: Record<string, unknown>) {
    if (typeof window !== "undefined") {
      localStorage.setItem("auth_token", token)
      localStorage.setItem("auth_user", JSON.stringify(user))
    }
  }

  getUserData(): { token: string | null; user: Record<string, unknown> | null } {
    if (typeof window === "undefined") {
      return { token: null, user: null }
    }
    const token = localStorage.getItem("auth_token")
    const userStr = localStorage.getItem("auth_user")
    const user = userStr ? JSON.parse(userStr) : null
    return { token, user }
  }
}

export const apiClient = new ApiClient(API_BASE_URL)
export type { ApiError, TokenResponse }
