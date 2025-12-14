"use client"

import { useState, useEffect } from "react"
import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Loading } from "@/components/loading"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { ScrollArea } from "@/components/ui/scroll-area"
import { FileCheck, Clock, AlertCircle, CheckCircle2 } from "lucide-react"
import { apiClient } from "@/lib/api-client"
import type { PolicyWithStats } from "@/types"

export default function PendingPoliciesPage() {
  const [policies, setPolicies] = useState<PolicyWithStats[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedPolicy, setSelectedPolicy] = useState<PolicyWithStats | null>(null)
  const [acknowledging, setAcknowledging] = useState(false)

  useEffect(() => {
    loadPendingPolicies()
  }, [])

  const loadPendingPolicies = async () => {
    try {
      setLoading(true)
      const data = await apiClient.get<PolicyWithStats[]>("/policies/pending")
      setPolicies(data)
    } catch (error) {
      console.error("Failed to load pending policies:", error)
    } finally {
      setLoading(false)
    }
  }

  const acknowledgePolicy = async (policyId: string) => {
    try {
      setAcknowledging(true)
      await apiClient.post(`/policies/${policyId}/acknowledge`, {})
      // Remove from list after acknowledgment
      setPolicies(prev => prev.filter(p => p.policy.id !== policyId))
      setSelectedPolicy(null)
    } catch (error) {
      console.error("Failed to acknowledge policy:", error)
      alert("Failed to acknowledge policy. Please try again.")
    } finally {
      setAcknowledging(false)
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <Loading />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Pending Policy Acknowledgments"
        description="Review and acknowledge the policies below to confirm you have read and understood them."
      />

      {policies.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <CheckCircle2 className="h-16 w-16 text-green-500 mb-4" />
            <h3 className="text-lg font-semibold mb-2">All Caught Up!</h3>
            <p className="text-muted-foreground text-center max-w-md">
              You have acknowledged all required policies. Check back later for any new or updated policies that may require your attention.
            </p>
          </CardContent>
        </Card>
      ) : (
        <>
          <div className="flex items-center gap-2 p-4 bg-amber-50 dark:bg-amber-950 rounded-lg border border-amber-200 dark:border-amber-800">
            <AlertCircle className="h-5 w-5 text-amber-600 dark:text-amber-400" />
            <span className="text-sm text-amber-800 dark:text-amber-200">
              You have {policies.length} {policies.length === 1 ? "policy" : "policies"} pending acknowledgment.
            </span>
          </div>

          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {policies.map((item) => (
              <Card key={item.policy.id} className="hover:border-primary/50 transition-colors">
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between">
                    <div className="space-y-1">
                      <CardTitle className="text-lg">{item.policy.title}</CardTitle>
                      <CardDescription className="font-mono text-xs">
                        {item.policy.code}
                      </CardDescription>
                    </div>
                    <Badge variant="outline">v{item.policy.version || 1}</Badge>
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  {item.policy.category && (
                    <Badge variant="secondary">{item.policy.category}</Badge>
                  )}

                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Clock className="h-4 w-4" />
                    {item.policy.effective_date ? (
                      <span>Effective: {new Date(item.policy.effective_date).toLocaleDateString()}</span>
                    ) : (
                      <span>Published</span>
                    )}
                  </div>

                  <Button
                    className="w-full"
                    onClick={() => setSelectedPolicy(item)}
                  >
                    <FileCheck className="mr-2 h-4 w-4" />
                    Review & Acknowledge
                  </Button>
                </CardContent>
              </Card>
            ))}
          </div>
        </>
      )}

      {/* Policy Review Dialog */}
      <Dialog open={!!selectedPolicy} onOpenChange={() => setSelectedPolicy(null)}>
        <DialogContent className="max-w-3xl max-h-[90vh]">
          {selectedPolicy && (
            <>
              <DialogHeader>
                <DialogTitle>{selectedPolicy.policy.title}</DialogTitle>
                <DialogDescription className="flex items-center gap-2">
                  <span className="font-mono">{selectedPolicy.policy.code}</span>
                  <Badge variant="outline">Version {selectedPolicy.policy.version || 1}</Badge>
                  {selectedPolicy.policy.category && (
                    <Badge variant="secondary">{selectedPolicy.policy.category}</Badge>
                  )}
                </DialogDescription>
              </DialogHeader>

              <ScrollArea className="max-h-[50vh] pr-4">
                <div className="prose prose-sm dark:prose-invert max-w-none">
                  {selectedPolicy.policy.content ? (
                    <div
                      dangerouslySetInnerHTML={{ __html: selectedPolicy.policy.content }}
                    />
                  ) : (
                    <p className="text-muted-foreground italic">
                      No policy content available.
                    </p>
                  )}
                </div>
              </ScrollArea>

              <div className="bg-muted/50 p-4 rounded-lg">
                <p className="text-sm text-muted-foreground">
                  By clicking "Acknowledge", you confirm that you have read, understood, and agree to comply with this policy.
                </p>
              </div>

              <DialogFooter>
                <Button variant="outline" onClick={() => setSelectedPolicy(null)}>
                  Cancel
                </Button>
                <Button
                  onClick={() => acknowledgePolicy(selectedPolicy.policy.id)}
                  disabled={acknowledging}
                >
                  {acknowledging ? "Acknowledging..." : "Acknowledge Policy"}
                </Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}
