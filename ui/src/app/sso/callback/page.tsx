"use client"

import { Suspense, useEffect, useState, useRef } from "react"
import { useRouter, useSearchParams } from "next/navigation"
import { apiClient } from "@/lib/api-client"
import { useAuth } from "@/contexts/auth-context"

function SSOCallbackContent() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const { setAuthData } = useAuth()
  const [status, setStatus] = useState<"processing" | "success" | "error">("processing")
  const [error, setError] = useState<string | null>(null)
  const hasAttemptedExchange = useRef(false)

  useEffect(() => {
    const handleSSOCallback = async () => {
      // Prevent multiple exchanges of the same authorization code
      if (hasAttemptedExchange.current) {
        console.log("SSO Callback - Already attempted exchange, skipping")
        return
      }
      hasAttemptedExchange.current = true

      try {
        // Clear any existing auth data
        apiClient.clearUserData()

        // Get authorization code from URL
        let code = searchParams.get("code")
        console.log("SSO Callback - Authorization code from searchParams:", code)

        // Fallback for S3 static hosting
        if (!code && typeof window !== "undefined") {
          const urlParams = new URLSearchParams(window.location.search)
          code = urlParams.get("code")
          console.log("SSO Callback - Authorization code from window.location:", code)
        }

        if (!code) {
          console.error("SSO Callback - No authorization code found")
          setError("No authorization code provided")
          setStatus("error")
          return
        }

        // Exchange code for access token
        console.log("SSO Callback - Exchanging code for access token")
        const tokenData = await apiClient.exchangeSSOCode(code)
        console.log("SSO Callback - Token received")

        // Get user info
        let userInfo = null
        try {
          const userInfoData = await apiClient.getSSOUserInfo(tokenData.access_token)
          console.log("SSO Callback - User info received:", userInfoData)

          // Find most relevant role
          let userRoles: string[] = ["user"]
          if (userInfoData.roles && Array.isArray(userInfoData.roles)) {
            userRoles = userInfoData.roles as string[]
          }

          userInfo = {
            id: userInfoData.sub as string,
            email: userInfoData.email as string,
            name: ((userInfoData.email as string) || "").split("@")[0],
            roles: userRoles,
            organization_id: userInfoData.organization_id as string | undefined,
          }
        } catch (userInfoErr) {
          console.error("SSO Callback - Failed to fetch user info:", userInfoErr)

          // Fallback: decode JWT
          try {
            const tokenParts = tokenData.access_token.split(".")
            if (tokenParts.length === 3) {
              const payload = JSON.parse(atob(tokenParts[1].replace(/-/g, "+").replace(/_/g, "/")))
              userInfo = {
                id: payload.sub || "unknown",
                email: payload.email || "user@opengrc.local",
                name: payload.name || payload.email?.split("@")[0] || "User",
                roles: payload.roles || ["user"],
                organization_id: payload.organization_id,
              }
            }
          } catch (decodeErr) {
            console.warn("SSO Callback - Could not decode JWT:", decodeErr)
          }
        }

        // Final fallback
        if (!userInfo) {
          userInfo = {
            id: "sso-user",
            email: "user@opengrc.local",
            name: "User",
            roles: ["user"],
          }
        }

        // Store auth data
        setAuthData(tokenData.access_token, userInfo)
        setStatus("success")

        // Redirect to dashboard
        setTimeout(() => {
          router.push("/")
        }, 1000)
      } catch (err) {
        console.error("SSO callback error:", err)
        setError(err instanceof Error ? err.message : "SSO authentication failed")
        setStatus("error")
      }
    }

    handleSSOCallback()
  }, [searchParams, router, setAuthData])

  return (
    <div className="min-h-screen flex items-center justify-center bg-background">
      <div className="max-w-md w-full space-y-8 p-8">
        <div className="text-center">
          {status === "processing" && (
            <>
              <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-primary mx-auto mb-4"></div>
              <h2 className="text-2xl font-bold">Completing SSO Login...</h2>
              <p className="mt-2 text-muted-foreground">
                Please wait while we log you in.
              </p>
            </>
          )}

          {status === "success" && (
            <>
              <div className="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-green-100 dark:bg-green-900 mb-4">
                <svg className="h-8 w-8 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <h2 className="text-2xl font-bold">Login Successful!</h2>
              <p className="mt-2 text-muted-foreground">
                Redirecting to dashboard...
              </p>
            </>
          )}

          {status === "error" && (
            <>
              <div className="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-red-100 dark:bg-red-900 mb-4">
                <svg className="h-8 w-8 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </div>
              <h2 className="text-2xl font-bold">SSO Login Failed</h2>
              <p className="mt-2 text-red-600 dark:text-red-400">
                {error || "An error occurred during SSO authentication"}
              </p>
              <button
                onClick={() => window.location.href = "/"}
                className="mt-4 px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90 transition-colors"
              >
                Try Again
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  )
}

export default function SSOCallbackPage() {
  return (
    <Suspense fallback={
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-primary"></div>
      </div>
    }>
      <SSOCallbackContent />
    </Suspense>
  )
}
