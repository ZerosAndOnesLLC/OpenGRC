"use client"

import { useAuth } from "@/contexts/auth-context"
import { useRouter, usePathname } from "next/navigation"
import { useEffect } from "react"
import config from "@/lib/config"

interface ProtectedRouteProps {
  children: React.ReactNode
}

export function ProtectedRoute({ children }: ProtectedRouteProps) {
  const { user, isLoading, isAuthenticated } = useAuth()
  const router = useRouter()
  const pathname = usePathname()

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      // Build SSO login URL with redirect back to current page
      const currentUrl = typeof window !== "undefined"
        ? window.location.origin + pathname
        : ""

      // Redirect to TitaniumVault SSO login
      const ssoUrl = `${config.auth.ssoLoginUrl}/login?client_id=${config.auth.clientId}&redirect_uri=${encodeURIComponent(currentUrl.replace(pathname, "/sso/callback/"))}`

      window.location.href = ssoUrl
    }
  }, [isLoading, isAuthenticated, router, pathname])

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    )
  }

  if (!user) {
    return null
  }

  return <>{children}</>
}
