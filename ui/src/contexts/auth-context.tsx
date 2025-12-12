"use client"

import React, { createContext, useContext, useState, useEffect } from "react"
import { apiClient } from "@/lib/api-client"

interface User {
  id: string
  email: string
  name: string
  roles: string[]
  organization_id?: string
}

interface AuthContextType {
  user: User | null
  token: string | null
  isLoading: boolean
  isAuthenticated: boolean
  logout: () => void
  setAuthData: (token: string, user: User) => void
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [token, setToken] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    const initAuth = async () => {
      try {
        const { token: savedToken, user: savedUser } = apiClient.getUserData()

        if (savedToken && savedUser) {
          setToken(savedToken)
          // Validate user data has required fields
          const user = savedUser as Record<string, unknown>
          if (user.id && user.email && user.name && user.roles) {
            setUser({
              id: user.id as string,
              email: user.email as string,
              name: user.name as string,
              roles: user.roles as string[],
              organization_id: user.organization_id as string | undefined,
            })
          }
        }
      } catch (error) {
        console.error("Failed to initialize auth:", error)
        apiClient.clearUserData()
      } finally {
        setIsLoading(false)
      }
    }

    initAuth()
  }, [])

  const logout = () => {
    apiClient.clearUserData()
    setUser(null)
    setToken(null)
  }

  const setAuthData = (newToken: string, newUser: User) => {
    apiClient.setUserData(newToken, newUser as unknown as Record<string, unknown>)
    setToken(newToken)
    setUser(newUser)
  }

  const value: AuthContextType = {
    user,
    token,
    isLoading,
    isAuthenticated: !!user && !!token,
    logout,
    setAuthData,
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider")
  }
  return context
}
